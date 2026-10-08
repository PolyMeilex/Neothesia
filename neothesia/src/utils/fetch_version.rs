#![allow(unused)]

fn get_latest() -> Option<String> {
    let body = ureq::get("https://api.github.com/repos/PolyMeilex/Neothesia/releases/latest")
        .call()
        .ok()?
        .body_mut()
        .read_to_string()
        .ok()?;

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
