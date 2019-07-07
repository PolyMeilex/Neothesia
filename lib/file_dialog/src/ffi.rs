extern "C" {
    pub fn free(p: *mut std::ffi::c_void);
}

#[repr(u32)]
pub enum MessageLevel {
    INFO = 0,
    WARNING = 1,
    ERROR = 2,
}
#[repr(u32)]
pub enum MessageButtons {
    OK = 0,
    OK_CANCEL = 1,
    YES_NO = 2,
}
#[repr(u32)]
pub enum FileAction {
    OPEN,
    DIR,
    SAVE,
}

extern "C" {
    pub fn osdialog_strndup(
        s: *const ::std::os::raw::c_char,
        n: usize,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    #[doc = " Launches a message box."]
    #[doc = ""]
    #[doc = "Returns 1 if the \"OK\" or \"Yes\" button was pressed."]
    pub fn osdialog_message(
        level: MessageLevel,
        buttons: MessageButtons,
        message: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " Launches an input prompt with an \"OK\" and \"Cancel\" button."]
    #[doc = ""]
    #[doc = "`text` is the default string to fill the input box."]
    #[doc = ""]
    #[doc = "Returns the entered text, or NULL if the dialog was cancelled."]
    #[doc = "If the returned result is not NULL, caller must free() it."]
    #[doc = ""]
    #[doc = "TODO: Implement on Windows and GTK2."]
    pub fn osdialog_prompt(
        level: MessageLevel,
        message: *const ::std::os::raw::c_char,
        text: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
}

#[repr(C)]
#[derive(Debug)]
pub struct filter_patterns {
    pub pattern: *mut ::std::os::raw::c_char,
    pub next: *mut filter_patterns,
}

#[repr(C)]
#[derive(Debug)]
pub struct filters {
    pub name: *mut ::std::os::raw::c_char,
    pub patterns: *mut filter_patterns,
    pub next: *mut filters,
}

extern "C" {
    #[doc = " Launches a file dialog and returns the selected path or NULL if nothing was selected."]
    #[doc = ""]
    #[doc = "`path` is the default folder the file dialog will attempt to open in, or NULL for the OS's default."]
    #[doc = "`filename` is the default text that will appear in the filename input, or NULL for the OS's default. Relevant to save dialog only."]
    #[doc = "`filters` is a list of patterns to filter the file selection, or NULL."]
    #[doc = ""]
    #[doc = "Returns the selected file, or NULL if the dialog was cancelled."]
    #[doc = "If the return result is not NULL, caller must free() it."]
    pub fn osdialog_file(
        action: FileAction,
        path: *const ::std::os::raw::c_char,
        filename: *const ::std::os::raw::c_char,
        filters: *mut filters,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    #[doc = " Parses a filter string."]
    #[doc = "Example: \"Source:c,cpp,m;Header:h,hpp\""]
    #[doc = "Caller must eventually free with osdialog_filters_free()."]
    pub fn osdialog_filters_parse(str: *const ::std::os::raw::c_char) -> *mut filters;
}
extern "C" {
    pub fn osdialog_filters_free(filters: *mut filters);
}

#[repr(C)]
pub struct color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

extern "C" {
    #[doc = " Launches an RGBA color picker dialog and sets `color` to the selected color."]
    #[doc = "Returns 1 if \"OK\" was pressed."]
    #[doc = ""]
    #[doc = "`color` should be set to the initial color before calling. It is only overwritten if the user selects \"OK\"."]
    #[doc = "`opacity` enables the opacity slider by setting to 1. Not supported on Windows."]
    #[doc = ""]
    #[doc = "TODO Implement on Mac."]
    pub fn osdialog_color_picker(
        color: *mut color,
        opacity: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
