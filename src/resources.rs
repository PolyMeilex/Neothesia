pub fn default_sf2() -> PathBuff {
    #[cfg(not(target_os = "macos"))]
    return std::path::Path::new("./default.sf2");
    #[cfg(target_os = "macos")]
    return bundled_resource_path("default", "sf2")
        .unwrap_or(std::path::Path::new("./default.sf2"));
}

pub fn settings_ron() -> PathBuff {
    #[cfg(not(target_os = "macos"))]
    return std::path::Path::new("./settings.ron");
    #[cfg(target_os = "macos")]
    return bundled_resource_path("settings", "ron")
        .unwrap_or(std::path::Path::new("./settings.ron"));
}

#[cfg(target_os = "macos")]
fn bundled_resource_path(name: &str, extension: &str) -> Option<String> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let cls = Class::get("NSBundle").unwrap();
        let bundle: *mut Object = msg_send![cls, mainBundle];
        let cls = Class::get("NSString").unwrap();
        let objc_str: *mut Object = msg_send![cls, alloc];
        let objc_name: *mut Object = msg_send![objc_str,
                                              initWithBytes:name.as_ptr()
                                              length:name.len()
                                              encoding: 4]; // UTF8_ENCODING
        let objc_str: *mut Object = msg_send![cls, alloc];
        let objc_ext: *mut Object = msg_send![objc_str,
                                              initWithBytes:extension.as_ptr()
                                              length:extension.len()
                                              encoding: 4]; // UTF8_ENCODING
        let ini: *mut Object = msg_send![bundle,
                                         pathForResource:objc_name
                                         ofType:objc_ext];
        let _: () = msg_send![objc_name, release];
        let _: () = msg_send![objc_ext, release];
        let cstr: *const i8 = msg_send![ini, UTF8String];
        if cstr != std::ptr::null() {
            let rstr = std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned();
            return Some(rstr);
        }
        None
    }
}
