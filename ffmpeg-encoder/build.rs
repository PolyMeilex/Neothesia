fn main() {
    for (name, value) in std::env::vars() {
        // This is documented at: https://github.com/zmwangx/rust-ffmpeg-sys#feature-flags
        let Some(version) = name.strip_prefix("DEP_FFMPEG_FFMPEG_") else {
            continue;
        };

        let flag = format!("ffmpeg_{}", version.to_lowercase());
        println!("cargo:rustc-check-cfg=cfg({flag})");

        if value == "true" {
            println!("cargo:rustc-cfg={flag}");
        }
    }
}
