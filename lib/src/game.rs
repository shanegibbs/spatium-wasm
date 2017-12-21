use ndarray::{Array, Ix2, Ix3};
use std::fmt;
use {SpatiumSys, SpatiumSysHelper};

pub struct Sprite {
    pub x: usize,
    pub y: usize,
}

impl Sprite {
    fn touches(&self, other: &Sprite) -> bool {
        self.x == other.x && self.y == other.y
    }
}

fn sprite(x: usize, y: usize) -> Sprite {
    Sprite { x: x, y: y }
}

#[derive(Debug)]
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

#[derive(Clone, Debug)]
pub struct GameState {
    pub arr: Array<u8, Ix2>,
}

pub struct Game {
    pub step: usize,
    pub width: usize,
    pub height: usize,
    pub done: bool,
    reward: usize,
    pub agent: Sprite,
    pub blocks: Vec<Sprite>,
    pub food: Vec<Sprite>,
}

impl Game {
    pub fn new() -> (Game, GameState, usize, bool) {
        let mut n = Game {
            step: 0,
            width: 3,
            height: 3,
            done: false,
            reward: 0,
            agent: sprite(0, 0),
            blocks: vec![sprite(1, 1)],
            food: vec![sprite(2, 2)],
        };
        let state = n.update_state();
        n.step = 0;
        (n, state.0, state.1, state.2)
    }
    fn build_state(&self) -> GameState {
        let mut state: Array<u8, Ix2> = Array::zeros((self.height, self.width));

        state[[self.agent.y, self.agent.x]] = 1;

        GameState { arr: state }
    }
    fn update_state(&mut self) -> (GameState, usize, bool) {
        // win
        let mut win = false;
        for foo in &self.food {
            if self.agent.touches(foo) {
                win = true;
                break;
            }
        }
        if win {
            self.reward += 10;
            self.done = true;
        }

        self.step += 1;
        (self.build_state(), self.reward, self.done)
    }
    pub fn step<T: SpatiumSys>(
        &mut self,
        helper: SpatiumSysHelper<T>,
        action: &Action,
    ) -> (GameState, usize, bool) {
        if self.done {
            panic!("Game already done");
        }

        let mut new_x = self.agent.x;
        let mut new_y = self.agent.y;

        match *action {
            Action::Up => {
                if self.agent.y <= 0 {
                    return self.update_state();
                }
                new_y -= 1;
            }
            Action::Right => {
                if self.agent.x >= self.width - 1 {
                    return self.update_state();
                }
                new_x += 1;
            }
            Action::Down => {
                if self.agent.y >= self.height - 1 {
                    return self.update_state();
                }
                new_y += 1;
            }
            Action::Left => {
                if self.agent.x <= 0 {
                    return self.update_state();
                }
                new_x -= 1;
            }
        }

        // unable to move
        let mut blocked = false;
        for block in &self.blocks {
            if block.x == new_x && block.y == new_y {
                blocked = true;
                break;
            }
        }
        if blocked {
            return self.update_state();
        }

        self.agent.x = new_x;
        self.agent.y = new_y;

        return self.update_state();
    }
}
