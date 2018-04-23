use super::neural_net::*;
use super::*;
use rng::RcRng;

use ndarray::prelude::*;
use rand::distributions::IndependentSample;
use rand::{self, Rng};

#[derive(Clone)]
struct Experience {
    state: GameState,
    action: Action,
    reward: f32,
    next_state: GameState,
    done: bool,
}

pub struct SingleLayerNetwork {
    parameters: SingleLayerNetworkParameters,
    step: usize,
    inputs: usize,
    outputs: usize,
    weights: Weights,
    target_weights: Weights,
    last_action: (f32, Array1<f32>, f32),
    explore_chance: f32,
    ep_numer: usize,
    experience_buf: Vec<Experience>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicValue {
    pub initial_rate: f32,
    pub final_rate: f32,
    pub final_episode: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleLayerNetworkParameters {
    pub minibatch_size: usize,
    pub expierence_buffer_size: usize,
    pub discount_factor: f32,
    pub learning: DynamicValue,
    pub exploration: DynamicValue,
}

impl Default for SingleLayerNetworkParameters {
    fn default() -> Self {
        SingleLayerNetworkParameters {
            minibatch_size: 10,
            expierence_buffer_size: 1000,
            discount_factor: 0.9,
            learning: DynamicValue {
                initial_rate: 0.001,
                final_rate: 0.001,
                final_episode: 1000,
            },
            exploration: DynamicValue {
                initial_rate: 1.0,
                final_rate: 0.1,
                final_episode: 1000,
            },
        }
    }
}

impl SingleLayerNetwork {
    pub fn new(parameters: SingleLayerNetworkParameters, ios: (usize, usize), rng: RcRng) -> Self {
        let (inputs, outputs) = ios;
        let weights = Weights::new(inputs, outputs, 32, parameters.learning.initial_rate, rng);

        SingleLayerNetwork {
            parameters,
            step: 0,
            inputs,
            outputs,
            weights: weights.clone(),
            target_weights: weights,
            last_action: (0., Array1::zeros(0), 0.),
            explore_chance: 1.0,
            ep_numer: 1,
            experience_buf: vec![],
        }
    }

    fn run_network(
        &self,
        _sys: &SpatiumSys,
        x_val: Array2<f32>,
        weights: &Weights,
    ) -> (Array1<f32>, Array2<f32>, Array1<f32>) {
        let net: NeuralNet = weights.into();
        net.run(x_val)
    }

    // returns a_val [len], q_val [len,4], max_q [len]
    fn run_q_network(
        &self,
        sys: &SpatiumSys,
        x_val: Array2<f32>,
    ) -> (Array1<f32>, Array2<f32>, Array1<f32>) {
        self.run_network(sys, x_val, &self.weights)
    }

    // returns a_val [len], q_val [len,4], max_q [len]
    fn run_target_network(
        &self,
        sys: &SpatiumSys,
        x_val: Array2<f32>,
    ) -> (Array1<f32>, Array2<f32>, Array1<f32>) {
        self.run_network(sys, x_val, &self.target_weights)
    }

    fn run_update(&mut self, x_val: Array2<f32>, y_val: Array2<f32>) -> f32 {
        let mut net: NeuralNet = (&self.weights).into();
        let e = net.update(x_val, y_val);
        self.weights = net.build_weights();
        e
    }
    fn update_variables(&mut self) {
        let ep_numer = self.ep_numer as f32;

        // update explore chance
        {
            let start_ex = self.parameters.exploration.initial_rate;
            let end_ex = self.parameters.exploration.final_rate;
            let final_ex_ep = self.parameters.exploration.final_episode;
            if self.ep_numer > final_ex_ep {
                self.explore_chance = end_ex;
            } else {
                let per_frame_loss = (start_ex - end_ex) / final_ex_ep as f32;
                let ex = start_ex - (ep_numer * per_frame_loss);
                self.explore_chance = ex;
            }
            // println!("expore_chance: {}", self.explore_chance);
        }

        // update learning rate
        {
            let start_lr = self.parameters.learning.initial_rate;
            let end_lr = self.parameters.learning.final_rate;
            let final_lr_ep = self.parameters.learning.final_episode;
            if self.ep_numer > final_lr_ep {
                self.weights.lr = end_lr;
            } else {
                let per_frame_loss = (start_lr - end_lr) / final_lr_ep as f32;
                let ex = start_lr - (ep_numer * per_frame_loss);
                self.weights.lr = ex;
            }
            // println!("lr: {}", self.sgd_lr);
        }
    }
}

impl Network for SingleLayerNetwork {
    fn test(&self, sys: &SpatiumSys, game_state: &GameState) -> (Action, f32) {
        let result = self.run_q_network(sys, game_state.into());
        ((result.0[0] as usize).into(), result.2[0] * 10.)
    }

    fn next_action(
        &mut self,
        sys: &SpatiumSys,
        rng: Option<RcRng>,
        s: &GameState,
    ) -> (Action, f32) {
        let mut performed = false;

        // exploring
        if let Some(mut rng) = rng {
            if rng.next_f32() < self.explore_chance {
                let dist = rand::distributions::Range::new(0, 3);
                self.last_action.0 = dist.ind_sample(&mut rng) as f32;
                performed = true;
            }
        }

        // greedy if not exporing
        if !performed {
            let result = self.run_q_network(sys, s.into());
            self.last_action = (result.0[0], result.1.row(0).to_owned(), result.2[0]);
        }

        (
            (self.last_action.0 as usize).into(),
            self.last_action.2 * 10.,
        )
    }
    fn result(
        &mut self,
        sys: &SpatiumSys,
        mut rng: RcRng,
        s: GameState,
        a: &Action,
        s1: &GameState,
        r: usize,
        done: bool,
    ) -> Metrics {
        let mut metrics: Metrics = Default::default();

        let experience_buf_size = self.parameters.expierence_buffer_size;
        let minibatch_size = self.parameters.minibatch_size;
        let discount_factor = self.parameters.discount_factor;

        // update expierence buffer
        if self.experience_buf.len() == experience_buf_size - 1 {
            sys.info("Expierence buffer full");
            metrics.annotations.push("Expierence buffer full".into());
        }
        self.experience_buf.push(Experience {
            state: s.clone(),
            action: *a,
            reward: r as f32,
            next_state: s1.to_owned(),
            done: done,
        });
        if self.experience_buf.len() > experience_buf_size {
            self.experience_buf.remove(0);
        }

        if self.experience_buf.len() >= experience_buf_size {
            let mut batch_states: Array<f32, Ix2> = Array::zeros((minibatch_size, self.inputs));
            let mut batch_targets: Array<f32, Ix2> = Array::zeros((minibatch_size, self.outputs));

            for i in 0..minibatch_size {
                let ex = rng.choose(&self.experience_buf).unwrap().to_owned();

                let reward = if ex.reward > 0. { 1. } else { 0. };
                let s1: Array2<f32> = (&ex.state).into();
                let s2: Array2<f32> = (&ex.next_state).into();

                // Combine s1 and s2 into states to run the target network
                // Update batch_states with s1 for the final update
                let mut states: Array2<f32> = Array::zeros((2, self.inputs));
                for n in 0..s1.shape()[1] {
                    states[[0, n]] = s1[[0, n]];
                    batch_states[[i, n]] = s1[[0, n]];
                }
                for n in 0..s2.shape()[1] {
                    states[[1, n]] = s2[[0, n]];
                }

                let result = self.run_target_network(sys, states);

                // we want our q1 to be the max of q2
                let q1_val = result.1.select(Axis(0), &[0]);
                let r2 = result.2[[1]];

                let mut target_q = q1_val;
                assert_eq!(target_q.shape(), &[1, 4]);

                let action_i: usize = (&ex.action).into();

                // println!("target: {:?}", result);
                // println!("ex.action: {:?}", ex.action);
                // println!("target_q_0: {:?}", target_q);
                let change_q = (discount_factor * r2) - target_q[[0, action_i]];
                target_q[[0, action_i]] = target_q[[0, action_i]] + (0.5 * change_q);
                if ex.done {
                    target_q[[0, action_i]] = reward;
                }
                // println!("target_q_1: {:?}", target_q);
                // println!("change_q: {}", change_q);

                for n in 0..target_q.shape()[1] {
                    batch_targets[[i, n]] = target_q[[0, n]];
                }

                // if ex.done {
                //     println!("TERMINAL {}", reward);
                // }
            }

            // println!("batch_states: {:?}", batch_states);
            // println!("batch_targets: {:?}", batch_targets);

            // let net: NeuralNet = (&self.weights).into();
            // let es = net.mse_each(batch_states.clone(), batch_targets.clone());
            // println!("es: {:?}", es);

            let e = self.run_update(batch_states, batch_targets);
            println!("e={}", e);
        }

        if self.step % 300 == 0 {
            self.target_weights = self.weights.clone();
        }

        if done {
            self.ep_numer += 1;
            self.update_variables();
        }

        self.step += 1;

        metrics
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::weak_rng;
    use spatium::tests::SpatiumDummy;

    #[test]
    fn test_main() {
        let dummy = SpatiumDummy {};
        let rng = RcRng::new(Box::new(weak_rng()));
        let mut net = SingleLayerNetwork::new(Default::default(), (9, 4), rng.clone());

        let mut state: ArrayD<u8> = Array::zeros(IxDyn(&[3, 3]));
        state[[1, 1]] = 1;
        let state = GameState { arr: state };

        // let input = state.map(|x| *x as f32).into_shape(IxDyn(&[1, 9])).unwrap();
        // let pred = net.make_prediction(input);
        // println!("{:?}", pred);

        let _a = net.next_action(&dummy, Some(rng), &state);
        // println!("{:?}", a);
    }
}
