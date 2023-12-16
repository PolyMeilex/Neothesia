use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{Arc, OnceLock},
};

fn config() -> Arc<rustls::ClientConfig> {
    static CONFIG: OnceLock<Arc<rustls::ClientConfig>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let root_store = rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            };

            rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()
                .into()
        })
        .clone()
}

pub struct Response {
    raw: String,
}

impl Response {
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn body(&self) -> Option<&str> {
        let mut split = self.raw.split("\r\n");
        for segment in &mut split {
            if segment.is_empty() {
                break;
            }
        }
        split.next()
    }
}

pub fn req(addr: impl ToSocketAddrs, host: &str, req: &str) -> Response {
    let example_com = host.to_string().try_into().unwrap();
    let client = rustls::ClientConnection::new(config(), example_com).unwrap();
    let conn = TcpStream::connect(addr).unwrap();

    let mut tls = rustls::StreamOwned::new(client, conn);
    tls.write_all(req.as_bytes()).unwrap();

    let mut plaintext = Vec::new();
    tls.read_to_end(&mut plaintext).unwrap();
    let raw = String::from_utf8(plaintext).unwrap();

    Response { raw }
}
