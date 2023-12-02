
## Usage

Install
```
pip3 install cargo-lambda==1.0.0
```

Deploy runtime
```
cargo lambda build --release
op-aws cargo lambda deploy --enable-function-url yoshino-radio
```

Deploy extension
```
cargo lambda build --release --extension
op-aws cargo lambda deploy --extension yoshino-radio
```

1. Copy the extension ARN 

```
op inject -i .env.production.tpl -o .env.production
```
1. Paste to the Layer in the function on AWS Console *with version*
* e.g. `arn:aws:lambda:us-west-2:12345678:layer:yoshino-radio:1`

https://www.youtube.com/watch?v=sAgUcJOwElU

https://docs.aws.amazon.com/lambda/latest/dg/rust-logging.html
