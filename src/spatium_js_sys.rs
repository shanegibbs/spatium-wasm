use spatium_lib::SpatiumSys;
use externs;

pub struct SpatiumJsSys;

impl SpatiumJsSys {
    pub fn new() -> SpatiumJsSys {
        SpatiumJsSys {}
    }
}

impl SpatiumSys for SpatiumJsSys {
    fn info(&self, s: &str) {
        externs::print(format!("[info] {}", s).as_str())
    }
    fn random(&mut self) -> f64 {
        externs::random()
    }
    fn draw_sprite(&self, i: usize, x: usize, y: usize) {
        externs::draw_sprite(i as u32, x as u32, y as u32)
    }
    fn clear_screen(&self) {
        externs::clear_screen()
    }
    fn frame_info(&self, info: &str) {
        externs::frame_info(info);
    }
    fn episode_number(&self, i: usize) {
        externs::episode_number(i as u32)
    }
}
