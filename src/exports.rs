use std::mem;
use std::os::raw::c_void;
use std::os::raw::*;
use std::ffi::CString;

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
pub extern "C" fn setup(max_episodes: usize) {
    ::setup(max_episodes)
}

#[no_mangle]
pub extern "C" fn step() -> *mut c_char {
    CString::new(::step()).unwrap().into_raw()
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