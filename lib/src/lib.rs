extern crate autograd as ag;
// #[macro_use(array)]
extern crate ndarray;
extern crate ndarray_rand;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod action;
mod game;
mod network;
mod rng;

pub use rng::RcRng;

use game::{Game, GameState, RenderingInfo};
use action::*;
use network::*;
pub use network::ModelParameters;
pub use network::model_descriptions;

use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeResult {
    steps: usize,
    score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub annotations: Vec<String>,
    pub values: Vec<(String, f32)>,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            annotations: vec![],
            values: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepResult {
    pub global_step: usize,
    pub episode: usize,
    pub step: usize,
    pub action: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_result: Option<EpisodeResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendering_info: Option<RenderingInfo>,
    pub metrics: Option<Metrics>,
}

pub trait SpatiumSys {
    fn debug(&self, &str) {}
    fn info(&self, s: &str) {
        println!("{}", s);
    }
    fn fatal(&self, e: &str) {
        panic!(format!("[fatal] {}", e))
    }
    fn random(&mut self) -> f64;
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
    fn debug<S: Into<String>>(&self, s: S) {
        self.sys.read().unwrap().debug(s.into().as_ref())
    }
}

pub struct Spatium<T: SpatiumSys> {
    sys: SpatiumSysHelper<T>,
    global_step: usize,
    episode: usize,
    max_episodes: usize,
    step: usize,
    network: Box<Network + Send>,
    game: Option<Game>,
    last_state: Option<(GameState, usize, bool)>,
    metrics: Option<Metrics>,
}

impl<T: SpatiumSys> Spatium<T> {
    pub fn new<P: IntoModelParameters>(
        raw_parameters: P,
        sys: T,
        rng: RcRng,
        max_episodes: usize,
    ) -> Result<Spatium<T>, String> {
        let model_params = raw_parameters.into_model_parameters()?;
        sys.info(&format!("Parsed model params: {:?}", model_params));
        
        let n = Spatium {
            sys: SpatiumSysHelper::new(sys),
            global_step: 0,
            step: 0,
            network: model_params.to_model(rng, 9, 4),
            episode: 0,
            max_episodes: max_episodes,
            game: None,
            last_state: None,
            metrics: None,
        };
        n.sys.info("Running Spatium");
        Ok(n)
    }
    fn is_final_state(&self) -> bool {
        self.last_state.as_ref().map(|s| s.2).unwrap_or(false)
    }
    fn do_final_frame(&mut self) -> EpisodeResult {
        let game = self.game.take().unwrap();
        self.last_state = None;
        self.episode += 1;
        self.step = 0;

        let sys = self.sys.read();

        self.sys.debug(format!(
            "Episode {} complete at step {}",
            self.episode, game.step
        ));

        let states = vec![
            (0, 0, Action::Right),
            (0, 1, Action::Right),
            (1, 0, Action::Down),
            (0, 2, Action::Down),
            (2, 0, Action::Right),
            (1, 2, Action::Down),
            (2, 1, Action::Right),
        ];

        if game.step < 40 && false {
            let mut score = 0;
            for (y, x, a) in states {
                if (y == 1 && x == 1) || (y == 2 && x == 2) {
                    continue;
                }
                let mut s = GameState {
                    arr: ndarray::Array::zeros((3, 3)),
                };
                s.arr[[y, x]] = 1;
                let (action, val) = self.network.next_action(&*sys, None, &s);
                let mut good = action == a;
                if y == 0 && x == 0 && action == Action::Down {
                    good = true;
                }
                if good {
                    score += 1;
                }
                println!(
                    "({},{}) = {:?} ({:?}), {} - val={}",
                    y, x, action, a, good, val
                );
            }
            println!("Score: {}", score);
        }

        EpisodeResult {
            steps: game.step,
            score: game.step as f32,
        }
    }
    fn reset_game(&mut self) {
        let (game, s, r, done) = Game::new(40);
        self.game = Some(game);
        self.last_state = Some((s, r, done));
        self.step += 1;
    }
    fn execute_action(&mut self, mut game: Game, action: &Action) -> (GameState, usize, bool) {
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
        let (action, _val) = self.network.next_action(&*sys, Some(rng.clone()), &s);

        // render the current game and the decided action
        let (s1, r, done) = self.execute_action(game, &action);

        let metrics = self.network
            .result(&*sys, rng.clone(), s, &action, &s1, r, done);
        self.metrics = Some(metrics);
    }
    pub fn step(&mut self, rng: RcRng) -> StepResult {
        // render final state
        if self.is_final_state() {
            // returns false on end of final episode

            let episode_result = Some(self.do_final_frame());

            let global_step = self.global_step;
            self.global_step += 1;

            return StepResult {
                global_step: global_step,
                episode: self.episode,
                step: self.step,
                action: "DIR".into(),
                done: self.episode >= self.max_episodes,
                episode_result: episode_result,
                rendering_info: self.game.as_ref().map(|g| g.rendering_info()),
                metrics: self.metrics.take(),
            };
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

        let global_step = self.global_step;
        self.global_step += 1;

        StepResult {
            global_step: global_step,
            episode: self.episode,
            step: self.step,
            action: "DIR".into(),
            done: false,
            episode_result: None,
            rendering_info: self.game.as_ref().map(|g| g.rendering_info()),
            metrics: self.metrics.take(),
        }
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
        let parameters = ModelParameters::QNetwork(Default::default());
        let p: String = serde_json::to_string(&parameters).unwrap();
        println!("Model parameters: {}", p);

        let mut spat = Spatium::new(parameters, SpatiumDummy {}, rng.clone(), 300).unwrap();
        loop {
            let result = spat.step(rng.clone());
            // println!("{}", serde_json::to_string(&result).unwrap());
            if let Some(ref _ep_result) = result.episode_result {
                println!("{}", serde_json::to_string(&result).unwrap());
            }
            if result.done {
                break;
            }
        }
    }
}
