use ndarray::prelude::*;
use SpatiumSys;
use action::*;
use super::*;

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game1Parameters {
    pub max_steps: usize,
    pub size: usize,
    pub random: bool,
}

impl Default for Game1Parameters {
    fn default() -> Self {
        let size = 10;
        Game1Parameters {
            max_steps: size * 4,
            size: size,
            random: true,
        }
    }
}

pub struct Game1 {
    params: Game1Parameters,
    state: State,
}

struct State {
    max_steps: usize,
    width: usize,
    height: usize,
    random: bool,
    step: usize,
    agent: Sprite,
    blocks: Vec<Sprite>,
    food: Vec<Sprite>,
    reward: usize,
    done: bool,
}

impl State {
    fn new(p: &Game1Parameters, rng: &mut RcRng) -> Self {
        let mut state = State {
            max_steps: p.max_steps,
            width: p.size,
            height: p.size,
            random: p.random,
            step: 0,
            agent: sprite(0, 0),
            blocks: vec![sprite(1, 1)],
            food: vec![],
            reward: 0,
            done: false,
        };

        let food = if p.random {
            state.random_empty_space(rng)
        } else {
            sprite(state.width - 1, state.height - 1)
        };
        state.food.push(food);

        state
    }
    fn is_free(&self, sprite: &Sprite) -> bool {
        if sprite.touches(&self.agent) {
            return false;
        }
        let food = self.food.iter();
        let blocks = self.blocks.iter();
        for s in food.chain(blocks) {
            if sprite.touches(s) {
                return false;
            }
        }
        true
    }
    fn random_empty_space(&self, rng: &mut RcRng) -> Sprite {
        use rand::distributions::{IndependentSample, Range};
        let width_range = Range::new(0, self.width);
        let height_range = Range::new(0, self.height);

        loop {
            let sprite = Sprite {
                x: width_range.ind_sample(rng),
                y: height_range.ind_sample(rng),
            };
            if self.is_free(&sprite) {
                return sprite;
            }
        }
    }
    fn build_state(&self) -> GameState {
        let mut layers = 1;
        if self.random {
            layers += 1;
        }
        let mut state: ArrayD<u8> = Array::zeros(IxDyn(&[layers, self.height, self.width]));

        let mut idx = 0;
        state[[idx, self.agent.y, self.agent.x]] = 1;
        if self.random {
            idx += 1;
            for sprite in &self.food {
                state[[idx, sprite.y, sprite.x]] = 1;
            }
        }

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
    fn step(&mut self, sys: &SpatiumSys, action: &Action) -> (GameState, usize, bool) {
        if self.done {
            panic!("Game already done");
        }

        sys.debug(&format!("Game step {} to {}", self.step, self.step + 1));

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

impl Game1 {
    pub fn new(p: Game1Parameters, mut rng: RcRng) -> Box<Game + Send> {
        let state = State::new(&p, &mut rng);
        let n = Self {
            params: p,
            state: state,
        };
        Box::new(n)
    }
}

impl Game for Game1 {
    fn io(&self) -> (usize, usize) {
        let mut layers = 1;
        if self.params.random {
            layers += 1;
        }
        (self.state.width * self.state.height * layers, 4)
    }
    fn reset(&mut self, mut rng: RcRng) -> (GameState, usize, bool) {
        self.state = State::new(&self.params, &mut rng);
        // self.step = 0;
        // self.done = false;
        // self.reward = 0;
        // self.agent = sprite(0, 0);
        // self.blocks = vec![sprite(1, 1)];

        // if self.random_food {
        //     self.food = vec![self.random_empty_space(&mut rng)];
        // } else {
        //     self.food = vec![sprite(self.width - 1, self.height - 1)];
        // }

        self.state.update_state()
    }

    fn rendering_info(&self) -> RenderingInfo {
        let agent_layer = RenderingLayer {
            name: "agent".into(),
            points: vec![
                Point {
                    x: self.state.agent.x,
                    y: self.state.agent.y,
                },
            ],
        };
        let block_layer = RenderingLayer {
            name: "block".into(),
            points: self.state
                .blocks
                .iter()
                .map(|s| Point { x: s.x, y: s.y })
                .collect(),
        };
        let food_layer = RenderingLayer {
            name: "food".into(),
            points: self.state
                .food
                .iter()
                .map(|s| Point { x: s.x, y: s.y })
                .collect(),
        };

        RenderingInfo {
            width: self.state.width,
            height: self.state.height,
            layers: vec![food_layer, agent_layer, block_layer],
        }
    }

    fn step(&mut self, sys: &SpatiumSys, action: &Action) -> (GameState, usize, bool) {
        self.state.step(sys, action)
    }
}
