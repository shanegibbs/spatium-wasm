use std::os::raw::*;
use std::ffi::CString;

extern "C" {
    fn sp_print(_: *mut c_char);
    fn sp_random() -> f64;
    fn sp_clear_screen();
    fn sp_draw_sprite(i: c_uint, x: c_uint, y: c_uint);
    fn sp_frame_info(_: *mut c_char);
    fn sp_episode_number(_: c_uint);
}

pub fn print(s: &str) {
    let c_str = CString::new(s).unwrap();
    unsafe {
        sp_print(c_str.into_raw());
    }
}

pub fn random() -> f64 {
    unsafe { sp_random() }
}

pub fn clear_screen() {
    unsafe { sp_clear_screen() }
}

pub fn draw_sprite(i: u32, x: u32, y: u32) {
    unsafe { sp_draw_sprite(i, x, y) }
}

pub fn frame_info(s: &str) {
    let c_str = CString::new(s).unwrap();
    unsafe {
        sp_frame_info(c_str.into_raw());
    }
}

pub fn episode_number(i: u32) {
    unsafe { sp_episode_number(i) }
}
