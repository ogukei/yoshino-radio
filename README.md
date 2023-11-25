
## Usage

```
pip3 install cargo-lambda==1.0.0
cd <this-repo>
cargo lambda build --release
op-aws cargo lambda deploy --enable-function-url
```


```
op inject -i .env.production.tpl -o .env.production
```


https://docs.aws.amazon.com/lambda/latest/dg/rust-logging.html