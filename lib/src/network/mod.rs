use serde;
use serde_json as json;

use game::GameState;
use action::*;
use SpatiumSys;
use Metrics;

use rng::RcRng;

mod qtable;
mod single_layer;

pub trait Network {
    fn next_action(&mut self, &SpatiumSys, Option<RcRng>, &GameState) -> (Action, f32);
    fn result(
        &mut self,
        &SpatiumSys,
        RcRng,
        GameState,
        &Action,
        &GameState,
        usize,
        bool,
    ) -> Metrics;
}

pub fn model_descriptions() -> Models {
    Models {
        q_table: ModelDescription {
            id: "QTable".into(),
            name: "Q-Table".into(),
            default_parameters: (),
        },
        q_network: ModelDescription {
            id: "QNetwork".into(),
            name: "Q-Network".into(),
            default_parameters: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Models {
    pub q_table: ModelDescription<()>,
    pub q_network: ModelDescription<single_layer::SingleLayerNetworkParameters>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDescription<P: serde::Serialize> {
    pub id: String,
    pub name: String,
    pub default_parameters: P,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModelParameters {
    QTable,
    QNetwork(single_layer::SingleLayerNetworkParameters),
}

pub trait IntoModelParameters {
    fn into_model_parameters(self) -> Result<ModelParameters, String>;
}

impl IntoModelParameters for ModelParameters {
    fn into_model_parameters(self) -> Result<ModelParameters, String> {
        Ok(self)
    }
}

impl<'a> IntoModelParameters for &'a str {
    fn into_model_parameters(self) -> Result<ModelParameters, String> {
        json::from_str(self).map_err(|e| format!("{}. String was:\n{}", e, self))
    }
}

impl ModelParameters {
    pub fn to_model(self, rng: RcRng, inputs: usize, outputs: usize) -> Box<Network + Send> {
        match self {
            ModelParameters::QTable => Box::new(qtable::QTable::new()),
            ModelParameters::QNetwork(p) => Box::new(single_layer::SingleLayerNetwork::new(
                p,
                inputs,
                outputs,
                rng,
            )),
        }
    }
}
