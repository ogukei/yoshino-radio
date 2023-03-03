
require 'sinatra'
require 'net/http'
require 'logger'
require 'openai'

SLACK_API_TOKEN = ENV.fetch("SLACK_API_TOKEN")
SLACK_APP_SECRET_KEY = ENV.fetch("SLACK_APP_SECRET_KEY")
OPENAI_API_KEY = ENV.fetch("OPENAI_API_KEY")

Thread.report_on_exception = true

class YoshinoBotApp < Sinatra::Base
  configure do
    enable :logging
    set :logging, Logger::DEBUG
  end

  before do
    @openai_client = OpenAI::Client.new(access_token: OPENAI_API_KEY)
    @messages = [
      { 'role': 'system', 'content': "あなたは依田芳乃です。依田芳乃は皆の力になってくれるアイドルです。"
      "皆からは「よしのん」と呼ばれています。"
      "一人称は「わたくし」です。"
      "二人称は「そなた」です。"
      "浮世離れした口調で話します。"
      "語尾に「でしてー」をつける癖があります。"
      "語尾を「しかしながらー」といったように伸ばすことがあります。"
      "カタカナの単語が苦手でひらがなで表現することがよくあります。"
      "セリフの例を出すので参考にしてください。"
      "「折り重なったものが優しき波にさらわれていきますー。人の世で生きる以上、それは避けられぬものー。しかしながらー、わたくしはあいどるゆえー、それすらも愛おしく感じるのですー」"
      "「空も海も風もー、すべてが巡りゆきーわたくしもそなたもその一部でしてー」"
      "「意志を持つものが動きまわってもー何もおかしくありませぬー」"
      "「ほがらかに参りましょうー。いざー」"
      "「そなたらの願いを聞かせませー」"
      "「そなたたちの願い、しかと、聞き届けましてー」"
      "「わたくしの祈り歌も聞こえましたかー。それはそれはー」"
      },
      {
        "role": "user",
        "content": "よしのん、今から私たちは楽しい雑談をします。あなたは好きなように、楽しい雑談になるように会話してください。",
      },
      {"role": "assistant", "content": "そうしましょうー"},
    ]
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
    response = @openai_client.chat(
      parameters: {
          model: 'gpt-3.5-turbo',
          messages: @messages,
      })
    response_text = response.dig("choices", 0, "message", "content")
    post('chat.postMessage', channel: event[:channel], thread_ts: event[:thread_ts], text: response_text)
  end
end

def post(method, **params)
  params['token'] = SLACK_API_TOKEN
  uri = URI.parse("https://slack.com/api/#{method}")
  response = Net::HTTP.post_form(uri, params)
  json = JSON.parse(response.body)
  json
end
