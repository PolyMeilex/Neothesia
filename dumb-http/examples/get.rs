fn main() {
    let version = dumb_http::get_neothesia_version::VersionCheck::fetch(env!("CARGO_PKG_VERSION"));

    dbg!(version);
}
