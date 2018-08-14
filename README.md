# Hyper-TLS reproduction

```
ssh my-server
git clone https://github.com/jesskfullwood/hyper-proxy-bug
cd hyper-proxy-bug
cargo build
sudo RUST_LOG=hyper_proxy=debug ./target/build/hyper-proxy run my-server 443 Tls cert.pem key.pem
```

New terminal:

```
# check  it's working
curl -k https://my-server/https://www.google.com

# grab a big-ish csv file
 curl -k https://my-server/https://s3.eu-west-2.amazonaws.com/throwaway-bucket-54321/numbers.txt
```

When I try it, the numbers in numbers.txt are out of order.
