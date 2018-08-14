# Hyper-TLS reproduction

```
ssh my-server
git clone https://github.com/jesskfullwood/hyper-proxy-bug
cd hyper-proxy-bug
cargo build
sudo RUST_LOG=hyper_proxy=debug ./target/debug/hyper-proxy 0.0.0.0 443 Tls cert.pem key.pem
```

New terminal:

```
# check  it's working
curl -k https://my-server/https://www.google.com

# grab a big-ish text file
 curl -k https://my-server/https://s3.eu-west-2.amazonaws.com/throwaway-bucket-54321/numbers.txt
 # also try with out the s     ^
```

When I try it, the numbers in numbers.txt are out of order (They should count from 0 - 999999)
