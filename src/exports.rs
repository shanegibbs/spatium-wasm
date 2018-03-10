use std::mem;
use std::os::raw::c_void;
use std::sync::Mutex;
use rand::SeedableRng;

use spatium_lib::{RcRng, Spatium};
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

fn random_u64() -> u64 {
    use std::u64::MAX;
    use externs::random;
    (random() * MAX as f64) as u64
}

fn rng() -> RcRng {
    use pcg_rand::Pcg32Basic;
    let rng = Pcg32Basic::from_seed([random_u64(), random_u64()]);
    RcRng::new(Box::new(rng))
}

lazy_static! {
  static ref DATA: Mutex<Spatium<SpatiumJsSys>> = 
    Mutex::new(Spatium::new(rng(), SpatiumJsSys::new(), 250));
}

#[no_mangle]
pub extern "C" fn step() -> bool {
    // let mut s = Spatium::new(rng(), SpatiumJsSys::new(), 250);
    // s.step(rng());
    // true
    DATA.lock().unwrap().step(rng())
}
