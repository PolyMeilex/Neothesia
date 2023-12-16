fn get_latest() -> Option<String> {
    let req = concat!(
        "GET /repos/PolyMeilex/Neothesia/releases/latest HTTP/1.1\r\n",
        "Host: api.github.com\r\n",
        "Connection: close\r\n",
        "Accept-Encoding: identity\r\n",
        "User-Agent: PostmanRuntime\r\n",
        "\r\n"
    );

    let addr = "api.github.com:443";
    let host = "api.github.com";

    let res = dumb_http::req(addr, host, req);

    let body = res.body().unwrap_or_default();

    let tag = "\"tag_name\":";

    let rest = &body[body.find(tag)? + tag.len()..];
    let rest = &rest[rest.find('"')? + 1..];
    let rest = &rest[..rest.find('"')?];

    Some(rest.to_string())
}

#[derive(Debug)]
pub struct VersionCheck {
    latest: Option<String>,
    current: &'static str,
}

impl VersionCheck {
    pub fn fetch() -> Self {
        Self {
            latest: get_latest(),
            current: env!("CARGO_PKG_VERSION"),
        }
    }

    pub fn latest(&self) -> Option<&str> {
        self.latest.as_deref()
    }

    pub fn is_latest(&self) -> bool {
        self.latest()
            .map(|latest| latest.contains(self.current))
            .unwrap_or(true)
    }
}
