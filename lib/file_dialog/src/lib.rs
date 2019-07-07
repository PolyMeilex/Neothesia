mod ffi;
use std::ffi::{c_void, CStr, CString};

use std::io::{Error, ErrorKind};

pub struct FileDialog<'a> {
    path: Option<&'a str>,
    action: Option<ffi::FileAction>,
    filters: Option<Vec<&'a str>>,
}

impl<'a> FileDialog<'a> {
    pub fn new() -> Self {
        FileDialog {
            path: None,
            filters: None,
            action: None,
        }
    }
    pub fn path(&'a mut self, path: &'a str) -> &mut FileDialog {
        self.path = Some(path);
        self
    }
    // pub fn action(&'a mut self, action: ffi::FileAction) -> &mut FileDialog {
    //     self.action = Some(action);
    //     self
    // }
    pub fn filters(&'a mut self, filters: Vec<&'a str>) -> &mut FileDialog {
        self.filters = Some(filters);
        self
    }
    pub fn open(&self) -> Result<String, Error> {
        let path = match self.path {
            Some(p) => CString::new(p).unwrap().as_ptr(),
            None => std::ptr::null(),
        };

        let filter = match &self.filters {
            Some(f) => {
                let filter = format!("Type:{}", f.join(","));
                let filter = CString::new(filter).unwrap();
                let filter = filter.as_ptr();
                unsafe { ffi::osdialog_filters_parse(filter) }
            }
            None => std::ptr::null_mut(),
        };


        unsafe {
            let c_buf = ffi::osdialog_file(ffi::FileAction::OPEN, path, std::ptr::null(), filter);

            ffi::osdialog_filters_free(filter);

            let res = c_buf.as_ref();

            let res = match res {
                Some(path) => Ok(CStr::from_ptr(path).to_str().unwrap().to_owned()),
                _ => Err(Error::new(ErrorKind::NotFound, "Dialog Canceled")),
            };
            ffi::free(c_buf as *mut c_void);
            return res;
        };
    }
}

// pub struct ColorPicker {
//     color: ffi::color,
//     has_opacity: bool,
// }

// impl ColorPicker {
//     pub fn new() -> Self {
//         ColorPicker {
//             color: ffi::color {
//                 r: 255,
//                 g: 255,
//                 b: 255,
//                 a: 255,
//             },
//             has_opacity: true,
//         }
//     }
// }

// pub fn color_test() {
//     let mut color = Box::new(ffi::color {
//         r: 255,
//         g: 0,
//         b: 5,
//         a: 255,
//     });
//     unsafe {
//         ffi::osdialog_color_picker(&mut *color, 0);
//     }
//     let color = Box::leak(color);
//     println!("{},{},{},{}", color.r, color.g, color.b, color.a);
// }