use std::mem;
use std::os::raw::c_void;
use std::sync::Mutex;

use spatium_lib::Spatium;
use spatium_js_sys::SpatiumJsSys;

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

lazy_static! {
  static ref DATA: Mutex<Spatium<SpatiumJsSys>> = Mutex::new(Spatium::new(SpatiumJsSys::new()));
}

// pub extern "C" fn get_charstar() -> *mut c_char {

#[no_mangle]
pub extern "C" fn step() -> bool {
    // let sys = StatiumJsSys {};
    // let run_output = spatium_lib::run(&sys);

    // let mut s = Spatium::new(SpatiumJsSys {});
    // s.step();

    DATA.lock().unwrap().step()
    // CString::new(DATA.step()run_output.as_str()).unwrap().into_raw()
}
