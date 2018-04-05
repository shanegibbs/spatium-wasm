use std::os::raw::*;
use std::ffi::CString;

extern "C" {
    fn sp_print(_: *mut c_char);
    fn sp_random() -> f64;
}

pub fn print(s: &str) {
    let c_str = match CString::new(s) {
        Ok(s) => s,
        Err(e) => {
            // fingers corssed we don't fail the error message!
            print(&format!("print() failed: {}", e));
            panic!(e);
        }
    };
    unsafe {
        sp_print(c_str.into_raw());
    }
}

pub fn random() -> f64 {
    unsafe { sp_random() }
}
