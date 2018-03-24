use super::Network;
use game::GameState;
use action::*;
use rng::RcRng;
use SpatiumSys;

use ag;
use rand::{self, Rng};
use ndarray::prelude::*;
use ag::gradient_descent_ops::Optimizer;
use rand::distributions::IndependentSample;

pub fn new(inputs: usize, outputs: usize, rng: RcRng) -> SingleLayerNetwork {
    SingleLayerNetwork::new(inputs, outputs, rng)
}

struct Experience {
    state: GameState,
    action: Action,
    reward: f32,
    next_state: GameState,
}

pub struct SingleLayerNetwork {
    inputs: usize,
    outputs: usize,
    w: ArrayD<f32>,
    b: ArrayD<f32>,
    last_action: (f32, ArrayD<f32>, f32),
    explore_chance: f32,
    sgd_lr: f32,
    ep_numer: usize,
    experience_buf: Vec<Experience>,
}

impl SingleLayerNetwork {
    pub fn new(inputs: usize, outputs: usize, rng: RcRng) -> Self {
        let arr_rng = ag::ndarray_ext::ArrRng::new(rng.clone());

        let w = arr_rng.glorot_uniform(&[inputs, outputs]);
        let b = ag::ndarray_ext::zeros(&[1, outputs]);

        SingleLayerNetwork {
            inputs,
            outputs,
            w,
            b,
            last_action: (0., ArrayD::zeros(IxDyn(&[0])), 0.),
            explore_chance: 0.1,
            sgd_lr: 0.1,
            ep_numer: 1,
            experience_buf: vec![],
        }
    }
    fn run_network(&self, _sys: &SpatiumSys, x_val: ArrayD<f32>) -> (f32, ArrayD<f32>, f32) {
        let x = ag::placeholder(&[-1, self.inputs as isize]);
        let w = ag::variable(self.w.clone());
        let b = ag::variable(self.b.clone());
        let z = ag::matmul(&x, &w) + &b;
        let zz = ag::sigmoid(&z);
        let max = ag::reduce_max(&zz, &[0], false);
        let a = ag::argmax(&zz, 1, false);

        let result = ag::eval(&[&a, &zz, &max], &[(&x, &x_val)]);

        let a_val = result[0][[0]];
        let q_val = result[1].clone();
        let max_q = result[2][[0]];

        assert!(
            a_val < 4.,
            format!("Bad a_val: {:?}. q_val: {}", a_val, q_val)
        );
        if q_val.shape() != &[1, 4] {
            println!("x={}", x_val);
            println!("w={}", self.w);
            println!("b={}", self.b);
            panic!("Wrong q_val shape");
        }

        (a_val, q_val, max_q)
    }
    fn run_update(&mut self, x_val: ArrayD<f32>, y_val: ArrayD<f32>) {
        let inputs_i = self.inputs as isize;
        let outputs_i = self.outputs as isize;

        let x = ag::placeholder(&[-1, inputs_i]);
        let y = ag::placeholder(&[-1, outputs_i]);
        let w = ag::variable(self.w.clone());
        let b = ag::variable(self.b.clone());
        let z = ag::matmul(&x, &w) + &b;
        let zz = ag::sigmoid(&z);

        let e = ag::sub(&y, &zz);
        let es = ag::square(&e);
        let mse = ag::reduce_sum(&es, &[0], false);
        let mse = ag::reduce_sum(&mse, &[0], false);

        let params = [&w, &b];
        let grads = ag::grad(&[&mse], &params);

        use ag::ops::gradient_descent_ops::sgd::SGD;
        let mut sgd = SGD { lr: self.sgd_lr };
        let update_ops = sgd.compute_updates(&params, &grads);

        let feeds = &[(&x, &x_val), (&y, &y_val)];

        ag::run(&update_ops, feeds);

        let new_vars = ag::eval(&[&w, &b], &[]);
        self.w = new_vars[0].clone();
        self.b = new_vars[1].clone();
    }
}

impl Network for SingleLayerNetwork {
    fn next_action(
        &mut self,
        sys: &SpatiumSys,
        rng: Option<RcRng>,
        s: &GameState,
    ) -> (Action, f32) {
        self.last_action = self.run_network(sys, s.into());

        // exploring
        if let Some(mut rng) = rng {
            if rng.next_f32() < self.explore_chance {
                let dist = rand::distributions::Range::new(0, 3);
                self.last_action.0 = dist.ind_sample(&mut rng) as f32;
            }
        }

        (
            (self.last_action.0 as usize).into(),
            self.last_action.2 * 10.,
        )
    }
    fn result(
        &mut self,
        sys: &SpatiumSys,
        _rng: RcRng,
        s: GameState,
        a: &Action,
        s1: &GameState,
        r: usize,
        done: bool,
    ) {
        // update expierence buffer
        self.experience_buf.push(Experience {
            state: s.clone(),
            action: *a,
            reward: r as f32,
            next_state: s1.to_owned(),
        });
        if self.experience_buf.len() > 300 {
            self.experience_buf.remove(0);
        }

        let y = 0.99;

        let (a_val, q_val, _) = self.last_action.clone(); // self.run_network(sys, (&s).into());
        let (_, _, q1_max) = self.run_network(sys, s1.into());

        let mut target_q = q_val;
        assert_eq!(target_q.shape(), &[1, 4]);

        target_q[[0, a_val as usize]] = (r as f32 / 10.) + (y * q1_max);

        self.run_update((&s).into(), target_q);

        if done {
            self.ep_numer += 1;
            let ep_numer = self.ep_numer as f32;

            // update explore chance
            {
                let start_ex = 1.;
                let end_ex = 0.01;
                let final_ex_ep = 290;
                if self.ep_numer > final_ex_ep {
                    self.explore_chance = 0.;
                } else {
                    let per_frame_loss = (start_ex - end_ex) / final_ex_ep as f32;
                    let ex = start_ex - (ep_numer * per_frame_loss);
                    self.explore_chance = ex;
                }
                println!("{}", self.explore_chance);
            }

            // update learning rate
            {
                let start_lr = 0.1;
                let end_lr = 0.01;
                let final_lr_ep = 290;
                if self.ep_numer > final_lr_ep {
                    self.sgd_lr = 0.;
                } else {
                    let per_frame_loss = (start_lr - end_lr) / final_lr_ep as f32;
                    let ex = start_lr - (ep_numer * per_frame_loss);
                    self.sgd_lr = ex;
                }
                println!("{}", self.sgd_lr);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tests::SpatiumDummy;
    use rand::weak_rng;

    #[test]
    fn test_main() {
        let dummy = SpatiumDummy {};
        let rng = RcRng::new(Box::new(weak_rng()));
        let mut net = SingleLayerNetwork::new(9, 4, rng.clone());

        let mut state: Array<u8, Ix2> = Array::zeros((3, 3));
        state[[1, 1]] = 1;
        let state = GameState { arr: state };

        // let input = state.map(|x| *x as f32).into_shape(IxDyn(&[1, 9])).unwrap();
        // let pred = net.make_prediction(input);
        // println!("{:?}", pred);

        let _a = net.next_action(&dummy, Some(rng), &state);
        // println!("{:?}", a);
    }
}
