
## Usage

Install
```
pip3 install cargo-lambda==1.0.0
```

Deploy runtimes
```
cd ./web
make
make deploy
cd ..
cd ./worker
make deploy
```

Add lambda:Invoke execution role to the web runtime

```
op inject -i .env.production.tpl -o .env.production
```

https://docs.aws.amazon.com/lambda/latest/dg/rust-logging.html
