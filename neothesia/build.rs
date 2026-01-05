fn main() {
    #[cfg(target_os = "windows")]
    {
        use std::{env, path::PathBuf};

        let in_ico = "../assets/icon.ico";

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let out_ico = out_dir.join("icon.ico");

        std::fs::copy(in_ico, out_ico).unwrap();

        let out_manifest = out_dir.join("manifest.rc");

        let manifest = "neothesia_icon ICON \"icon.ico\"";
        std::fs::write(&out_manifest, manifest).unwrap();

        let _ = embed_resource::compile(&out_manifest, embed_resource::NONE);
    }
}
