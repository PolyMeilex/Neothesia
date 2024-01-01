fn main() {
    #[cfg(target_os = "windows")]
    {
        use image::io::Reader as ImageReader;
        use std::{
            env,
            fs::File,
            path::{Path, PathBuf},
        };

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let out_ico = out_dir.join("icon.ico");
        let out_manifest = out_dir.join("manifest.rc");

        let img = ImageReader::open("../flatpak/com.github.polymeilex.neothesia.png")
            .unwrap()
            .decode()
            .unwrap();

        img.save(&out_ico);

        let manifest = "neothesia_icon ICON \"icon.ico\"";
        std::fs::write(&out_manifest, &manifest);

        embed_resource::compile(&out_manifest, embed_resource::NONE);
    }
}
