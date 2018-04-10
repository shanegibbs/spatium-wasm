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

impl StepResult {
    fn new(
        episode: usize,
        step: usize,
        action: String,
        done: bool,
        rendering_info: RenderingInfo,
    ) -> Self {
        StepResult {
            global_step: 0,
            episode: episode,
            step: step,
            action: action,
            done: done,
            episode_result: None,
            rendering_info: Some(rendering_info),
            metrics: None,
        }
    }
    fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
    fn with_episode_result(mut self, episode_result: EpisodeResult) -> Self {
        self.episode_result = Some(episode_result);
        self
    }
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

struct RunningArgs {
    episode: usize,
    step: usize,
    game: Game,
    game_state: GameState,
}

enum EpisodeState {
    Init { episode: usize },
    Running(RunningArgs),
}

pub struct Spatium<T: SpatiumSys> {
    sys: SpatiumSysHelper<T>,
    max_episodes: usize,
    network: Box<Network + Send>,
    episode_state: Option<EpisodeState>,
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
            network: model_params.to_model(rng, 9, 4),
            episode_state: None,
            max_episodes: max_episodes,
        };
        n.sys.info("Running Spatium");
        Ok(n)
    }
    fn process_inital_state(&self, episode: usize) -> (EpisodeState, StepResult) {
        let (game, game_state, _score, _done) = Game::new(30);
        let rendering_info = game.rendering_info();
        (
            EpisodeState::Running(RunningArgs {
                episode: episode,
                step: 1,
                game: game,
                game_state: game_state,
            }),
            StepResult::new(episode, 0, "DIR".into(), false, rendering_info),
        )
    }
    fn process_running_state(
        &mut self,
        rng: RcRng,
        args: RunningArgs,
    ) -> (EpisodeState, StepResult) {
        let sys = self.sys.clone();
        let sys = sys.read();

        let RunningArgs {
            episode,
            step,
            mut game,
            game_state,
        } = args;

        // get next action from model
        let (action, _val) = self.network
            .next_action(&*sys, Some(rng.clone()), &game_state);

        // advance game using action
        let (game_state1, score1, done) = game.step(self.sys.clone(), &action);

        // pass result to model and collect any metrics
        let metrics = self.network.result(
            &*sys,
            rng.clone(),
            game_state,
            &action,
            &game_state1,
            score1,
            done,
        );

        let result = StepResult::new(
            episode,
            step,
            "DIR".into(),
            episode > self.max_episodes,
            game.rendering_info(),
        ).with_metrics(metrics);

        if done {
            self.sys
                .debug(format!("Episode {} complete at step {}", episode, step));
            (
                EpisodeState::Init {
                    episode: episode + 1,
                },
                result.with_episode_result(EpisodeResult {
                    steps: step,
                    score: step as f32,
                }),
            )
        } else {
            (
                EpisodeState::Running(RunningArgs {
                    episode: episode,
                    step: step + 1,
                    game: game,
                    game_state: game_state1,
                }),
                result,
            )
        }
    }
    pub fn step(&mut self, rng: RcRng) -> StepResult {
        let episode_state = self.episode_state.take();
        let (new_state, result) = match episode_state {
            None => {
                println!("No EpisodeState");
                self.process_inital_state(0)
            }
            Some(EpisodeState::Init { episode }) => {
                println!("EpisodeState::Init");
                self.process_inital_state(episode)
            }
            Some(EpisodeState::Running(args)) => {
                println!("EpisodeState::Running");
                self.process_running_state(rng, args)
            }
        };
        self.episode_state = Some(new_state);
        result
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;

    pub struct SpatiumDummy;

    impl SpatiumSys for SpatiumDummy {
        fn debug(&self, s: &str) {
            println!("debug: {}", s);
        }
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

        let mut spat = Spatium::new(parameters, SpatiumDummy {}, rng.clone(), 2).unwrap();
        loop {
            let result = spat.step(rng.clone());
            println!("{}", serde_json::to_string(&result).unwrap());
            // if let Some(ref _ep_result) = result.episode_result {
            //     println!("{}", serde_json::to_string(&result).unwrap());
            // }
            if result.done {
                break;
            }
        }
    }
}
