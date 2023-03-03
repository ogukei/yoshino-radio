# Yoshino Radio

A simple ChatGPT Slack integration in Ruby

依田芳乃のお悩み相談らじお

よしのんが相談に乗ってくれます

## Minimal Setup

### 1. PaaS Setup
Skip this step if you have a Herokuish PaaS environment already such as Dokku.

1. Install Heroku CLI
    * See https://devcenter.heroku.com/articles/heroku-cli

1. Create an app
    * See https://devcenter.heroku.com/articles/git

### 2. Slack Setup

1. Go to https://api.slack.com/apps and "Create New App"
1. In Features -> App Home
    * Configure "App Display Name"
1. In Show Tabs -> Messages Tab
    * Turn on "Messages Tab"
    * Turn on "Allow users to send Slash commands and messages from the messages tab"
1. In Features -> Event Subscriptions
    * Enter "Request URL"
        * Such as Heroku endpoint deployed from this repository.
        * e.g. `https://yoshino-radio.herokuapp.com/`
    * In "Subscribe to bot events", add `message.im` scope
1. In Features -> OAuth & Permissions
    * Add the following "Bot Token Scopes"s
        * `chat:write`
        * `im:history`
1. In Features -> OAuth & Permissions, execute "Install to Workspace"

### 3. Environment Variables Setup
Requires the following evironment variables.

* SLACK_API_TOKEN
    * In Features -> OAuth & Permissions, copy "Bot User OAuth Token"
    * https://api.slack.com/apps
* SLACK_APP_SECRET_KEY
    * In Basic Information -> App Credentials, copy "Signing Secret"
    * https://api.slack.com/apps
* OPENAI_API_KEY
    * Navigate to your OpenAI API console and create a new secret key
    * https://platform.openai.com/account/api-keys

```
heroku config:set SLACK_API_TOKEN=<SLACK_API_TOKEN>
heroku config:set SLACK_APP_SECRET_KEY=<SLACK_APP_SECRET_KEY>
heroku config:set OPENAI_API_KEY=<OPENAI_API_KEY>
heroku logs
```

### 4. Final Setup
Done Buooo. Open Slack and make sure you can now make DM conversations with the bot.
