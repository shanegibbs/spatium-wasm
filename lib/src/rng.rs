use rand::Rng;
use std::rc::Rc;
use std::cell::RefCell;

pub struct RcRng {
    rng: Rc<RefCell<Box<Rng>>>,
}

impl RcRng {
    pub fn new(rng: Box<Rng>) -> Self {
        RcRng {
            rng: Rc::new(RefCell::new(rng)),
        }
    }
}

impl Clone for RcRng {
    fn clone(&self) -> Self {
        RcRng {
            rng: self.rng.clone(),
        }
    }
}

impl Rng for RcRng {
    fn next_u32(&mut self) -> u32 {
        let mut rng = self.rng.borrow_mut();
        rng.next_u32()
    }
}
