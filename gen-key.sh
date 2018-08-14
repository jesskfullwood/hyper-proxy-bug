#!/bin/sh
# https://stackoverflow.com/questions/10175812/how-to-create-a-self-signed-certificate-with-openssl
openssl req -x509 -newkey rsa:4096 -keyout pkcs10key.pem -out cert.pem -days 365
openssl pkcs8 -in pkcs10key.pem -out key.pem
