use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Up,
    Right,
    Down,
    Left,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Action::Up => write!(f, "Up"),
            Action::Right => write!(f, "Right"),
            Action::Down => write!(f, "Down"),
            Action::Left => write!(f, "Left"),
        }
    }
}

impl From<usize> for Action {
    fn from(i: usize) -> Action {
        match i {
            0 => Action::Up,
            1 => Action::Right,
            2 => Action::Down,
            3 => Action::Left,
            i => panic!(format!("Bad action value: {}", i)),
        }
    }
}

impl<'a> From<&'a Action> for usize {
    fn from(a: &'a Action) -> usize {
        match *a {
            Action::Up => 0,
            Action::Right => 1,
            Action::Down => 2,
            Action::Left => 3,
        }
    }
}
