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
mod spatium;

pub use spatium::Spatium;
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
