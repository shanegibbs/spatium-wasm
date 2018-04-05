use std::mem;
use std::os::raw::c_void;
use std::os::raw::*;
use std::ffi::{CStr, CString};

use serde_json as json;
use spatium_lib;

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf); // This is JS' responsibility now
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut c_void, cap: usize) {
    unsafe {
        let _buf = Vec::from_raw_parts(ptr, 0, cap);
    }
}

#[no_mangle]
pub extern "C" fn version() -> usize {
    env!("BUILD_VERSION").parse().unwrap()
}

#[no_mangle]
pub extern "C" fn model_descriptions() -> *mut c_char {
    let s = json::to_string(&spatium_lib::model_descriptions()).unwrap();
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn setup(model_params: *mut c_char, max_episodes: usize) -> *mut c_char {
    let model_params = unsafe { CStr::from_ptr(model_params).to_string_lossy().into_owned() };
    CString::new(
        match ::setup(&model_params, max_episodes) {
            Ok(()) => json!({"result": "ok"}),
            Err(e) => json!({"result": "error", "message": e}),
        }.to_string(),
    ).unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn step(count: usize) -> *mut c_char {
    CString::new(::step(count)).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn eval(_state: usize) -> [f32; 4] {
    [0., 1., 0., 1.]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        version();
    }
}
