use serde;
use serde_json as json;
use ndarray::prelude::*;
use RcRng;

mod game1;

use super::SpatiumSys;
use Network;
pub use self::game1::Game1Parameters;
use action::Action;

pub trait Game {
    fn io(&self) -> (usize, usize);
    fn reset(&mut self, rng: RcRng) -> (GameState, usize, bool);
    fn step(&mut self, sys: &SpatiumSys, action: &Action) -> (GameState, usize, bool);
    fn rendering_info(&self) -> RenderingInfo;
    fn eval(&self, &SpatiumSys, &Box<Network + Send>);
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub arr: ArrayD<u8>,
}

impl<'a> Into<Array2<f32>> for &'a GameState {
    fn into(self) -> Array2<f32> {
        let x: Vec<_> = self.arr.iter().map(|n| *n as f32).collect();
        let l = x.len();
        Array::from_shape_vec([1, l], x).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderingInfo {
    width: usize,
    height: usize,
    layers: Vec<RenderingLayer>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderingLayer {
    name: String,
    points: Vec<Point>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    x: usize,
    y: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Games {
    pub game1: GameDescription<(usize)>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDescription<P: serde::Serialize> {
    pub id: String,
    pub name: String,
    pub default_parameters: P,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameParameters {
    Game1(Game1Parameters),
}

pub trait IntoGameParameters {
    fn into_parameters(self) -> Result<GameParameters, String>;
}

impl IntoGameParameters for GameParameters {
    fn into_parameters(self) -> Result<GameParameters, String> {
        Ok(self)
    }
}

impl<'a> IntoGameParameters for &'a str {
    fn into_parameters(self) -> Result<GameParameters, String> {
        json::from_str(self).map_err(|e| format!("{}. String was:\n{}", e, self))
    }
}

impl GameParameters {
    pub fn into_game(self, rng: RcRng) -> (Box<Game + Send>) {
        match self {
            GameParameters::Game1(p) => game1::Game1::new(p, rng),
        }
    }
}
