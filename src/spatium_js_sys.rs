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
    fn fatal(&self, s: &str) {
        externs::print(format!("[fatal] {}", s).as_str())
    }
    fn random(&mut self) -> f64 {
        externs::random()
    }
}
