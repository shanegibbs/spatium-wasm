extern crate autograd as ag;
#[macro_use(array)]
extern crate ndarray;
extern crate ndarray_rand;
extern crate rand;

mod action;
mod game;
mod network;
mod rng;

pub use rng::RcRng;

use game::{Game, GameState};
use action::*;
use network::*;

use std::sync::{Arc, RwLock, RwLockReadGuard};

pub trait SpatiumSys {
    fn info(&self, &str) {}
    fn random(&mut self) -> f64;
    fn clear_screen(&self) {}
    fn draw_sprite(&self, _i: usize, _x: usize, _y: usize) {}
    fn frame_info(&self, _info: &str) {}
    fn episode_number(&self, _i: usize) {}
}

pub struct SpatiumSysHelper<T: SpatiumSys> {
    sys: Arc<RwLock<T>>,
}

impl<T> Clone for SpatiumSysHelper<T>
where
    T: SpatiumSys,
{
    fn clone(&self) -> Self {
        SpatiumSysHelper {
            sys: Arc::clone(&self.sys),
        }
    }
}

impl<T: SpatiumSys> SpatiumSysHelper<T> {
    fn new(t: T) -> SpatiumSysHelper<T> {
        SpatiumSysHelper {
            sys: Arc::new(RwLock::new(t)),
        }
    }
    fn read(&self) -> RwLockReadGuard<T> {
        self.sys.read().unwrap()
    }
    fn info<S: Into<String>>(&self, s: S) {
        self.sys.read().unwrap().info(s.into().as_ref())
    }
    // fn draw_sprite(&self, i: usize, x: usize, y: usize) {
    //     self.sys.read().unwrap().draw_sprite(i, x, y)
    // }
    // fn clear_screen(&self) {
    //     self.sys.read().unwrap().clear_screen()
    // }
}

pub struct Spatium<T: SpatiumSys> {
    sys: SpatiumSysHelper<T>,
    episode: usize,
    max_episodes: usize,
    step: usize,
    network: Box<Network + Send>,
    game: Option<Game>,
    last_state: Option<(GameState, usize, bool)>,
}

impl<T: SpatiumSys> Spatium<T> {
    pub fn new(rng: RcRng, sys: T, max_episodes: usize) -> Spatium<T> {
        let network: Box<Network + Send> = if "snn" == "qtable" {
            Box::new(network::qtable::new())
        } else {
            // Box::new(network::qtable::new())
            Box::new(network::single_layer::new(9, 4, rng))
        };
        let n = Spatium {
            sys: SpatiumSysHelper::new(sys),
            step: 0,
            network: network as Box<Network + Send>,
            episode: 0,
            max_episodes: max_episodes,
            game: None,
            last_state: None,
        };
        n.sys.info("Running Spatium");
        n
    }
    fn render(&self, game: &Game, action: Option<&Action>) {
        let sys = self.sys.read();

        sys.clear_screen();
        for s in &game.blocks {
            sys.draw_sprite(1, s.x, s.y);
        }
        for s in &game.food {
            sys.draw_sprite(2, s.x, s.y);
        }
        sys.draw_sprite(0, game.agent.x, game.agent.y);

        let action_str = action.map(|a| format!("{}", a)).unwrap_or(format!("None"));

        let frame_info = format!(
            "Episode: {}\nStep: {}\nDone: {}\nAction: {}",
            self.episode, game.step, game.done, action_str
        );
        sys.frame_info(frame_info.as_str());

        sys.episode_number(self.episode);
    }
    fn is_final_state(&self) -> bool {
        self.last_state.as_ref().map(|s| s.2).unwrap_or(false)
    }
    fn do_final_frame(&mut self) -> bool {
        let game = self.game.take().unwrap();
        self.render(&game, None);
        self.last_state = None;
        self.episode += 1;

        let sys = self.sys.read();
        sys.episode_number(self.episode);

        self.sys.info(format!(
            "Episode {} complete at step {}",
            self.episode, game.step
        ));

        let mut s = GameState { arr: ndarray::Array::zeros((3, 3)) };
        s.arr[[0, 0]] = 1;
        let action = self.network.next_action(&*sys, None, &s);
        println!("1 DR: {:?}", action);

        let mut s = GameState { arr: ndarray::Array::zeros((3, 3)) };
        s.arr[[0, 1]] = 1;
        let action = self.network.next_action(&*sys, None, &s);
        println!("2 R: {:?}", action);

        let mut s = GameState { arr: ndarray::Array::zeros((3, 3)) };
        s.arr[[0, 2]] = 1;
        let action = self.network.next_action(&*sys, None, &s);
        println!("3 D: {:?}", action);

        let mut s = GameState { arr: ndarray::Array::zeros((3, 3)) };
        s.arr[[1, 2]] = 1;
        let action = self.network.next_action(&*sys, None, &s);
        println!("4 D: {:?}", action);

        // check if this was the last episode
        if self.episode >= self.max_episodes {
            self.sys.info(format!("All episodes executed"));
            return false;
        }
        return true;
    }
    fn reset_game(&mut self) {
        let (game, s, r, done) = Game::new(40);
        self.game = Some(game);
        self.last_state = Some((s, r, done));
        self.step += 1;
    }
    fn execute_action(&mut self, mut game: Game, action: &Action) -> (GameState, usize, bool) {
        // render current state and new action
        self.render(&game, Some(&action));

        // step game using action
        let state = game.step(self.sys.clone(), &action);

        // prepare for next step
        self.game = Some(game);
        self.last_state = Some(state.clone());
        self.step += 1;

        state
    }
    // do AI stuff and call self.execute_action
    fn process(&mut self, rng: RcRng, game: Game, s: GameState) {
        let sys = self.sys.clone();
        let sys = sys.read();
        let action = self.network.next_action(&*sys, Some(rng.clone()), &s);

        // render the current game and the decided action
        let (s1, r, done) = self.execute_action(game, &action);

        self.network.result(&*sys, s, &action, &s1, r, done);
    }
    pub fn step(&mut self, rng: RcRng) -> bool {
        // render final state
        if self.is_final_state() {
            // returns false on end of final episode
            return self.do_final_frame();
        }

        // setup new game
        if self.last_state.is_none() {
            self.reset_game();
        }

        // extract current state
        let game = self.game.take().unwrap();
        let (s, _last_r, _) = self.last_state.take().unwrap();

        // process step
        self.process(rng, game, s);

        true
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;

    pub struct SpatiumDummy;

    impl SpatiumSys for SpatiumDummy {
        fn info(&self, s: &str) {
            println!("info: {}", s);
        }
        fn random(&mut self) -> f64 {
            rand::random()
        }
    }

    #[test]
    fn it_works() {
        let rng = RcRng::new(Box::new(rand::weak_rng()));
        let mut spat = Spatium::new(rng.clone(), SpatiumDummy {}, 300);
        while spat.step(rng.clone()) {}
    }
}
