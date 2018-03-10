use game::GameState;
use action::*;
use SpatiumSys;

use rng::RcRng;

pub mod qtable;
pub mod single_layer;

pub trait Network {
    fn next_action(&mut self, &SpatiumSys, Option<RcRng>, &GameState) -> Action;
    fn result(&mut self, &SpatiumSys, GameState, &Action, &GameState, usize, bool);
}
