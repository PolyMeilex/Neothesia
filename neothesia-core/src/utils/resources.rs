use std::{env, path::PathBuf};

fn home() -> Option<PathBuf> {
    env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

fn xdg_config() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
        .map(|p| p.join("neothesia"))
        .or_else(|| home().map(|h| h.join(".config").join("neothesia")))
}

pub fn default_sf2() -> Option<PathBuf> {
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    {
        if let Some(path) = xdg_config().map(|p| p.join("default.sf2"))
            && path.exists()
        {
            return Some(path);
        }

        // <prefix>/bin/neothesia -> <prefix>/share/neothesia/default.sf2
        if let Some(path) = std::env::current_exe().ok().and_then(|exe_path| {
            exe_path
                .parent()
                .and_then(|path| path.parent())
                .map(|pfx_path| pfx_path.join("share").join("neothesia").join("default.sf2"))
        }) && path.exists()
        {
            return Some(path);
        }

        // Development: workspace-root default.sf2 (debug builds only).
        #[cfg(debug_assertions)]
        if let Some(path) = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(|p| p.join("default.sf2"))
            && path.exists()
        {
            return Some(path);
        }

        let flatpak = PathBuf::from("/app/share/neothesia/default.sf2");
        if flatpak.exists() {
            Some(flatpak)
        } else {
            None
        }
    }

    #[cfg(target_os = "windows")]
    return Some(PathBuf::from("./default.sf2"));

    #[cfg(target_os = "macos")]
    return bundled_resource_path("default", "sf2").map(PathBuf::from);
}

pub fn settings_ron() -> Option<PathBuf> {
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    return xdg_config().map(|p| p.join("settings.ron"));

    #[cfg(target_os = "windows")]
    return Some(PathBuf::from("./settings.ron"));

    #[cfg(target_os = "macos")]
    return bundled_resource_path("settings", "ron").map(PathBuf::from);
}

#[cfg(target_os = "macos")]
fn bundled_resource_path(name: &str, extension: &str) -> Option<String> {
    use objc2_foundation::{NSBundle, NSString};

    let bundle = NSBundle::mainBundle();
    let name = NSString::from_str(name);
    let ext = NSString::from_str(extension);
    let path = bundle.pathForResource_ofType(Some(&name), Some(&ext))?;

    Some(path.to_string())
}
