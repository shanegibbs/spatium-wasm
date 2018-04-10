use ndarray::prelude::*;
use {SpatiumSys, SpatiumSysHelper};
use action::*;

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

#[derive(Clone, Debug)]
pub struct GameState {
    pub arr: Array<u8, Ix2>,
}

impl<'a> Into<Array2<f32>> for &'a GameState {
    fn into(self) -> Array2<f32> {
        let x = self.arr.iter().map(|n| *n as f32).collect();
        Array::from_shape_vec([1, 9], x).unwrap()
    }
}

pub struct Game {
    pub step: usize,
    pub max_steps: usize,
    pub width: usize,
    pub height: usize,
    pub done: bool,
    reward: usize,
    pub agent: Sprite,
    pub blocks: Vec<Sprite>,
    pub food: Vec<Sprite>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderingInfo {
    x: usize,
    y: usize,
}

impl Game {
    pub fn new(max_steps: usize) -> (Game, GameState, usize, bool) {
        let n = Game {
            step: 0,
            max_steps: max_steps,
            width: 3,
            height: 3,
            done: false,
            reward: 0,
            agent: sprite(0, 0),
            blocks: vec![sprite(1, 1)],
            food: vec![sprite(2, 2)],
        };
        let state = n.build_state();
        let reward = n.reward;
        let done = n.done;
        (n, state, reward, done)
    }
    pub fn rendering_info(&self) -> RenderingInfo {
        RenderingInfo {
            x: self.agent.x,
            y: self.agent.y,
        }
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
        if self.step >= self.max_steps {
            self.done = true;
        }

        (self.build_state(), self.reward, self.done)
    }
    pub fn step<T: SpatiumSys>(
        &mut self,
        sys: SpatiumSysHelper<T>,
        action: &Action,
    ) -> (GameState, usize, bool) {
        if self.done {
            panic!("Game already done");
        }

        sys.debug(format!("Game step {} to {}", self.step, self.step + 1));

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
