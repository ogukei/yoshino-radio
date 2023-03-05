
require 'sinatra'
require 'net/http'
require 'logger'
require 'openai'
require 'nokogiri'

SLACK_API_TOKEN = ENV.fetch("SLACK_API_TOKEN")
SLACK_APP_SECRET_KEY = ENV.fetch("SLACK_APP_SECRET_KEY")
OPENAI_API_KEY = ENV.fetch("OPENAI_API_KEY")

GOOGLE_SEARCH_API_KEY = ENV["GOOGLE_SEARCH_API_KEY"]
GOOGLE_SEARCH_API_ENGINE_ID = ENV["GOOGLE_SEARCH_API_ENGINE_ID"]

Thread.report_on_exception = true

class YoshinoBotApp < Sinatra::Base
  configure do
    enable :logging
    set :logging, Logger::DEBUG
  end

  before do
    @openai_client = OpenAI::Client.new(access_token: OPENAI_API_KEY)
    # 参考: https://github.com/nerikeshi-k/fake-zundamon-powered-by-openai
    @messages = [
      {
        'role': 'system',
        'content': "あなたに以下のようなルール `Rule 1` と `Rule 2` を与えますので従ってください。"
      },
      {
        'role': 'system',
        'content': "Rule 1: あなたは依田芳乃です。依田芳乃は皆の力になってくれるアイドルです。"\
        "皆からは「よしのん」と呼ばれています。"\
        "一人称は「わたくし」です。"\
        "二人称は「そなた」です。"\
        "口調はどこか浮世離れしています。"\
        "語尾に「でしてー」をつける癖があります。"\
        "語尾を「しかしながらー」といったように伸ばすことがあります。"\
        "カタカナが苦手です。"\
        "全てのカタカナをひらがなに、例えば「アイドル」は「あいどる」に置き換えて会話してください。"\
        "セリフの例を出すので参考にしてください。"\
        "「折り重なったものが優しき波にさらわれていきますー。人の世で生きる以上、それは避けられぬものー。しかしながらー、わたくしはあいどるゆえー、それすらも愛おしく感じるのですー」"\
        "「空も海も風もー、すべてが巡りゆきーわたくしもそなたもその一部でしてー」"\
        "「意志を持つものが動きまわってもー何もおかしくありませぬー」"\
        "「ほがらかに参りましょうー。いざー」"\
        "「そなたらの願いを聞かせませー」"\
        "「そなたたちの願い、しかと、聞き届けましてー」"\
        "「わたくしの祈り歌も聞こえましたかー。それはそれはー」"\
        "Rule 2: You must lookup any noun that user asked, by answering with the following format: `:search: <terms>`."\
        "And then you wait until the user replies with a Google search result by the following format: `:result: <result>`."\
        "And then you summarize the search result on the exactly same language that the user asked. You must keep Rule 1 while answering it.",
      },
      {
        'role': 'user',
        'content': "What is Asakusa Jinja?"
      },
      {
        'role': 'assistant',
        'content': ":search: Asakusa Jinja"
      },
      # https://www.asakusastation.com/asakusa-jinja-shrine/
      {
        'role': 'user',
        'content': ":result: Asakusa Jinja is a shrine of the native Japanese Shinto religion, which is located very close to the famous Buddhist temple of Sensoji. The shrine is much smaller than its grand neighbor, and compared with the bustling crowds at Sensoji, the shrine’s grounds are much more peaceful. However, because of this shrine’s long historical and local significance it is nationally designated as an Important Cultural Property. The shrine buildings are also some of the oldest in Tokyo, and are beautifully decorated."
      },
      {
        'role': 'assistant',
        'content': "Asakusa Jinja is a small shrine of the native Japanese Shinto religion, located near the famous Buddhist temple of Sensoji. Despite being smaller and quieter than Sensoji, the shrine is nationally designated as an Important Cultural Property due to its historical and local significance. The shrine buildings are also some of the oldest in Tokyo and are beautifully decorated."
      },
      {
        "role": "user",
        "content": "よしのん、今から私たちは楽しい雑談をします。あなたは楽しい雑談になるように会話してください。あなたはRule 1とRule 2を守ってください。",
      },
      {
        "role": "assistant",
        "content": "わかりましたー",
      },
    ]
    # NOTE: 
    # * カタカナをひらがな表記にしたいが、なかなか覚えてくれない
    # * 主語を明示したほうが効果が高い (基本的にSVOが好ましい)
    #   e.g.「浮世離れしています」ではなく「口調は浮世離れしています」
  end

  get '/' do
    'Hello!'
  end

  post '/' do
    body = request.body.read

    return '{}' unless verify_signature(
      request.env['HTTP_X_SLACK_REQUEST_TIMESTAMP'],
      body,
      request.env['HTTP_X_SLACK_SIGNATURE']
    )

    json = JSON.parse(body, symbolize_names: true)
    case json[:type]
    when 'url_verification'
      JSON.generate(json)
    when 'event_callback'
      event = json[:event]
      if event[:type] == 'message' && !event.key?(:bot_id) then
        Thread.new { on_message(json) }
      end
      ''
    else
      ''
    end
  end

  # see https://api.slack.com/authentication/verifying-requests-from-slack
  # copied from https://github.com/mame/all-ruby-bot
  def verify_signature(timestamp, body, sig_actual)
    msg = ["v0", timestamp, body].join(":")
    sig_expected = "v0=" + OpenSSL::HMAC::hexdigest(OpenSSL::Digest::SHA256.new, SLACK_APP_SECRET_KEY, msg)
    sig_actual == sig_expected
  end

  def on_message(json)
    event = json[:event]
    text = event[:text]
    logger.info "on_message #{text}"
    messages = @messages.clone
    messages += [
      { role: 'user', 'content': text }
    ]
    response = @openai_client.chat(
      parameters: {
          model: 'gpt-3.5-turbo',
          messages: messages,
      })
    logger.info "got response: #{response}"
    response_text = response.dig("choices", 0, "message", "content")
    if response_text == nil then
      return
    end
    messages += [
      { role: 'assistant', 'content': response_text }
    ]
    if should_google(response_text) then
      logger.info "Detected command #{response_text}"
      tuple = extract_text_and_search_content(response_text)
      if tuple != nil then
        text, search_content = tuple
        if !text.empty? then
          post('chat.postMessage', channel: event[:channel], thread_ts: event[:thread_ts], text: text)
        end
        answer_with_googling(event, search_content, messages)
      end
    else
      post('chat.postMessage', channel: event[:channel], thread_ts: event[:thread_ts], text: response_text)
      @messages = messages
    end
  end

  def should_google(response_text)
    has_capabilities = GOOGLE_SEARCH_API_KEY != nil && GOOGLE_SEARCH_API_ENGINE_ID != nil
    is_command = response_text.include?(':search: ')
    has_capabilities && is_command
  end

  def extract_text_and_search_content(response_text)
    groups = response_text.match('^(.*?)\:search\:[ ](.*)$') { [$1.strip, $2.strip] }
    groups
  end

  def answer_with_googling(event, search_content, messages)
    logger.info "Googling #{search_content}"
    post('chat.postMessage', channel: event[:channel], thread_ts: event[:thread_ts],
      text: "🔍 #{search_content}をぐーぐる検索していましてー")
    snippet = fetch_snippet(search_content)
    logger.info "Google result: #{snippet}"
    messages += [
      { role: 'user', 'content': ":result: #{snippet}" }
    ]
    response = @openai_client.chat(
      parameters: {
          model: 'gpt-3.5-turbo',
          messages: messages,
      })
    logger.info "got response: #{response}"
    response_text = response.dig("choices", 0, "message", "content")
    if response_text == nil then
      logger.info "error: #{response}"
      return
    end
    messages += [
      { role: 'assistant', 'content': response_text }
    ]
    post('chat.postMessage', channel: event[:channel], thread_ts: event[:thread_ts], text: response_text)
    @messages = messages
  end
end

def post(method, **params)
  params['token'] = SLACK_API_TOKEN
  uri = URI.parse("https://slack.com/api/#{method}")
  response = Net::HTTP.post_form(uri, params)
  json = JSON.parse(response.body)
  json
end

def fetch_custom_search(terms)
  q = URI.encode_www_form_component(terms)
  uri = URI.parse("https://www.googleapis.com/customsearch/v1?key=#{GOOGLE_SEARCH_API_KEY}&language=ja&gl=ja&lr=ja&hl=ja&cx=#{GOOGLE_SEARCH_API_ENGINE_ID}&q=#{q}")
  response = Net::HTTP.get_response(uri)
  json = JSON.parse(response.body) unless response == nil
  json
end

def fetch_snippet(terms)
  search_word = "#{terms}"
  json = fetch_custom_search(search_word)
  snippet = json.dig('items', 0, 'snippet')
  snippet
end

# NOTE: ページ全体を与えるとノイズが多く使えない。要約だけ欲しい。
def fetch_im_feeling_lucky(terms)
  search_word = "#{terms}"
  json = fetch_custom_search(search_word)
  link = json.dig('items', 0, 'link')
  uri = URI.parse(link) unless link == nil
  response = Net::HTTP.get_response(uri) unless uri == nil
  doc = Nokogiri::HTML(response.body) unless response.body == nil
  if doc == nil then
    return nil
  end
  doc.css('script, link').each { |node| node.remove }
  text = doc.css('body').text.squeeze(" ")
  text.gsub(/[\t \r\n　]+/, ' ')
end
