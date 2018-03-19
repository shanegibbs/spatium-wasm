use std::mem;
use std::os::raw::c_void;

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
pub extern "C" fn ping() -> usize {
    7
}

#[no_mangle]
pub extern "C" fn setup(max_episodes: usize) {
    ::setup(max_episodes)
}

#[no_mangle]
pub extern "C" fn step() -> bool {
    ::step()
}

#[no_mangle]
pub extern "C" fn eval(_state: usize) -> [f32; 4] {
    [0., 1., 0., 1.]
}
