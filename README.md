
# Yoshino Radio

A simple example of integrating AWS Lambda, Slack and ChatGPT in Rust

依田芳乃のお悩み相談らじお

よしのんが相談に乗ってくれます

## Minimal Setup

### 1. Install

* Install [Cargo Lambda](https://www.cargo-lambda.info/guide/getting-started.html)

```
pip3 install cargo-lambda==1.0.0
```

### 2. Build

Please make sure the build succeeds

```
cd <this-repo>
cd web
make

cd ..
cd worker
make
```

### 3. Slack Setup

1. Go to https://api.slack.com/apps and "Create New App"
1. In Features -> App Home
    * Configure "App Display Name"
1. In Show Tabs -> Messages Tab
    * Turn on "Messages Tab"
    * Turn on "Allow users to send Slash commands and messages from the messages tab"
1. In Features -> Event Subscriptions
    * Enter "Request URL"
        * described later in this README
    * In "Subscribe to bot events", add `message.im` scope
1. In Features -> OAuth & Permissions
    * Add the following "Bot Token Scopes"s
        * `chat:write`
        * `im:history`
1. In Features -> OAuth & Permissions, execute "Install to Workspace"

### 4. Setup Credentials

* Place `web/.env.production` env file with the following content
   * Find your Slack App's signing secret in the Basic Information > App Credentials
   * https://api.slack.com/apps

```
SLACK_SIGNING_SECRET=abcdef12345
```

* Place `worker/.env.production` env file with the following content
   * Find your Slack App's client token in the OAuth & Permissions > OAuth Tokens for Your Workspace
      * https://api.slack.com/apps
   * Find your OpenAI API key in the console
      * https://platform.openai.com/api-keys

```
SLACK_CLIENT_TOKEN=xoxb-12345
OPENAI_API_KEY=sk-12345
```

### 5. Deploy

You will need to set the `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` environment variables to deploy the executable to AWS Lambda.

See https://docs.aws.amazon.com/sdk-for-php/v3/developer-guide/guide_credentials_environment.html

#### Deploying Web Runtime

* Execute the following command

```
cd ./web
make deploy
```

* Copy the URL displayed on the console after running the command
* Paste it to the [Slack App](https://api.slack.com/apps)'s Request URL
    * In Features -> Event Subscriptions > "Request URL"
* Open AWS IAM and edit roles
* Attach the policy `AWSLambdaRole` to the execution role of the web runtime
   * The `lambda:InvokeFunction` action is required to invoke the worker runtime from the web runtime.
   * The execution role is automatically created by Cargo Lambda after first deployment
      * https://www.cargo-lambda.info/commands/deploy.html

#### Deploying Worker Runtime

```
cd ./worker
make deploy
```

### 6. Final Setup
Done Buooo. Open Slack and make sure you can now make DM conversations with the bot.
