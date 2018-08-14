extern crate futures;
extern crate http;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rustls;
#[macro_use]
extern crate structopt;
extern crate tokio_rustls;
extern crate tokio_tcp;

use futures::prelude::*;
use http::uri;
use http::HttpTryFrom;
use hyper::client::Client;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use hyper_tls::HttpsConnector;
use rustls::internal::pemfile;
use rustls::ServerConfig;
use structopt::StructOpt;
use tokio_rustls::ServerConfigExt;

use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[derive(StructOpt, Clone)]
struct Config {
    host: String,
    port: u16,
    #[structopt(subcommand)]
    policy: SecurityPolicy,
}

#[derive(StructOpt, Clone)]
enum SecurityPolicy {
    Http,
    Tls { certfile: String, keyfile: String },
}

fn logerr(e: impl ToString) -> String {
    let e = e.to_string();
    error!("{}", e);
    e
}

fn service(
    mut req: Request<hyper::Body>,
) -> impl Future<Item = Response<Body>, Error = String> + Send {
    info!("Request in: {:#?}", req);
    let uri = req.uri().clone();
    // TODO this fails when lacking a scheme and never gets fixed up
    uri::Uri::try_from(&uri.to_string()[1..])
        .map_err(logerr)
        .into_future()
  	    .and_then(move |new_uri| {
            {
            let headers = req.headers_mut();
            headers.remove("Host");
            if let Some(new_host) = new_uri.host() {
                headers.insert(
                    "Host",
                    hyper::header::HeaderValue::from_str(new_host)
                        .expect("HeaderValue failed to parse"),
                );
            }
            }
            let new_uri = fixup_uri(new_uri);
            let scheme = new_uri.scheme_part().cloned();
            *req.uri_mut() = new_uri;
            info!("Request out: {:#?}", req);
            match &scheme {
                Some(v) if *v == uri::Scheme::HTTP => {
                    let client = Client::new();
                    client.request(req).map_err(logerr)
                }
                Some(v) if *v == uri::Scheme::HTTPS => {
                    let https = HttpsConnector::new(4).expect("HttpsConnector failed");
                    let client = Client::builder().build::<_, hyper::Body>(https);
                    client.request(req).map_err(logerr)
                }
                // TODO should handle this error more gracefully
                other => panic!("Invalid scheme: {:?}", other),
            }
        })
        .map(move |resp| {
            info!("Response in: {:#?}", resp);
            resp
        })

    // UNCOMMENT TO FIX

        // .and_then(move |resp| {
        //     let (mut parts, body) = resp.into_parts();
        //     body.concat2().map(|whole_body| {
        //         let body = Body::from(whole_body);
        //         let respout = Response::from_parts(parts, body);
        //         info!("Response out: {:#?}", respout);
        //         respout
        //     }).map_err(logerr)
        // })
}

fn fixup_uri(url: uri::Uri) -> uri::Uri {
    debug!("{:?}", url);
    let mut parts = url.into_parts();
    if parts.scheme.is_none() {
        // Try https
        warn!("Missing scheme given - fallback to HTTPS");
        parts.scheme = Some(uri::Scheme::HTTPS);
    }
    if let None = parts.path_and_query {
        parts.path_and_query = Some(uri::PathAndQuery::from_static("/"));
    }
    uri::Uri::from_parts(parts).expect("Uri create failed")
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = File::open(filename).expect("cannot open certificate file");
    let mut certfile = BufReader::new(certfile);
    pemfile::certs(&mut certfile).unwrap()
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = File::open(filename).expect("cannot open private key file");
    let mut keyfile = BufReader::new(keyfile);
    let keys = pemfile::pkcs8_private_keys(&mut keyfile).expect("failed to parse key file");
    assert!(keys.len() == 1);
    keys[0].clone()
}

fn serve_http(port: u16) {
    let addr = ([0, 0, 0, 0], port).into();
    let server = Server::bind(&addr).serve(move || {
        service_fn(move |req| service(req))
    });
    hyper::rt::run(server.or_else(|e| {
        error!("server error: {}", e);
        Ok(())
    }));
}

fn serve_tls(port: u16, certfile: &str, keyfile: &str) {
    loop {
        let addr = ([0, 0, 0, 0], port).into();
        let tcp = tokio_tcp::TcpListener::bind(&addr).expect("TLS bind failed");

        let certs = load_certs(certfile);
        let key = load_private_key(keyfile);
        let mut cfg = ServerConfig::new(rustls::NoClientAuth::new());
        cfg.set_single_cert(certs, key).unwrap();
        let tls_cfg = Arc::new(cfg);

        let tls = tcp.incoming().and_then(move |s| tls_cfg.accept_async(s));
        let server = Server::builder(tls).serve(move || {
            service_fn(move |req| service(req))
        });

        hyper::rt::run(server.or_else(|e| {
            error!("server error: {}", e);
            Ok(())
        }));
        info!("Restarting server")
    }
}

fn main() {
    env_logger::init();
    let mut config = Config::from_args();
    {
        let host = &mut config.host;
        if host.ends_with("/") {
            let _ = host.pop();
        }
    }
    match config.policy {
        SecurityPolicy::Tls { certfile, keyfile } => {
            serve_tls(config.port, &certfile, &keyfile)
        }
        SecurityPolicy::Http => serve_http(config.port),
    }
}
