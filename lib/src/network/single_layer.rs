use super::Network;
use game::GameState;
use action::*;
use rng::RcRng;
use SpatiumSys;

use ag;
use rand;
use ndarray::prelude::*;
use ag::gradient_descent_ops::Optimizer;
use rand::distributions::IndependentSample;

pub fn new(inputs: usize, outputs: usize, rng: RcRng) -> SingleLayerNetwork {
    SingleLayerNetwork::new(inputs, outputs, rng)
}

pub struct SingleLayerNetwork {
    inputs: usize,
    outputs: usize,
    w: ArrayD<f32>,
    b: ArrayD<f32>,
    episode: Vec<(GameState, Action, f32)>,
}

impl SingleLayerNetwork {
    pub fn new(inputs: usize, outputs: usize, rng: RcRng) -> Self {
        let inputs = inputs + outputs; // 4 actions
        let outputs = 1; // output ` q-value

        let arr_rng = ag::ndarray_ext::ArrRng::new(rng.clone());

        let w = arr_rng.glorot_uniform(&[inputs, outputs]);
        let b = ag::ndarray_ext::zeros(&[1, outputs]);

        SingleLayerNetwork {
            inputs,
            outputs,
            w,
            b,
            episode: vec![],
        }
    }
    fn run_network(&self, _sys: &SpatiumSys, x_val: ArrayD<f32>) -> ArrayD<f32> {
        let inputs_i = self.inputs as isize;
        // let outputs_i = self.outputs as isize;

        let x = ag::placeholder(&[-1, inputs_i]);
        // let y = ag::placeholder(&[-1, outputs_i]);
        let w = ag::variable(self.w.clone());
        let b = ag::variable(self.b.clone());
        let z = ag::matmul(&x, &w) + &b;
        let zz = ag::sigmoid(&z);
        // let loss = ag::sparse_softmax_cross_entropy(&z, &y);
        // let ref loss_square = ag::square(loss);
        // let ref mse = ag::reduce_mean(loss_square, &[0], false);
        // let params = [&w, &b];
        // let grads = ag::grad(&[&loss], &params);
        // let predictions = ag::argmax(&z, -1, true);
        // let ref accuracy = ag::reduce_mean(&ag::equal(predictions, y), &[0], false);
        // let mut adam = ag::gradient_descent_ops::Adam::default();
        // let ref update_ops = adam.compute_updates(params, grads);
        // let update_ops =
        // ag::ops::gradient_descent_ops::sgd::SGD { lr: 0.01 }.compute_updates(&params, &grads);

        // let noise = ag::random_uniform(&[1., 4.], 0., 3., rng);
        // let zz_plus_noise = ag::add(&zz, &noise);
        // let predictions = ag::argmax(&zz_plus_noise, -1, true);

        let result = ag::eval(&[&zz], &[(&x, &x_val)]);
        // sys.info(&format!("zz            = {:?}", result[0]));
        result[0].clone()
    }
    fn run_update(&mut self, x_val: ArrayD<f32>, y_val: ArrayD<f32>) {
        let inputs_i = self.inputs as isize;
        let outputs_i = self.outputs as isize;
        // println!("{} -> {}", inputs_i, outputs_i);

        let x = ag::placeholder(&[-1, inputs_i]);
        let y = ag::placeholder(&[-1, outputs_i]);
        let w = ag::variable(self.w.clone());
        let b = ag::variable(self.b.clone());
        let z = ag::matmul(&x, &w) + &b;
        let zz = ag::sigmoid(&z);

        // let e = ag::sigmoid_cross_entropy(&z, &y);
        let e = ag::sub(&y, &zz);
        let es = ag::square(&e);
        let mse = ag::reduce_sum(&es, &[0], false);

        let params = [&w, &b];
        let grads = ag::grad(&[&mse], &params);

        use ag::ops::gradient_descent_ops::sgd::SGD;
        let mut sgd = SGD { lr: 0.1 };
        let update_ops = sgd.compute_updates(&params, &grads);

        let feeds = &[(&x, &x_val), (&y, &y_val)];

        // println!("Running update");
        // let result = ag::eval(&[&z, &e, &mse], feeds);
        // println!("zz={:?}", result[0]);
        // println!("loss={:?}", result[1]);
        // println!("mse={:?}", result[2]);

        // println!("Calculating grads");
        // let result = ag::eval(&[&grads[0]], feeds);
        // let result = grad.eval(feeds);
        // println!("grad_w_mse={:?}", result);

        println!("mse={:?}", mse.eval(feeds));
        ag::run(&update_ops, feeds);
        println!("mse={:?}", mse.eval(feeds));

        let new_vars = ag::eval(&[&w, &b], &[]);
        self.w = new_vars[0].clone();
        self.b = new_vars[1].clone();

        // println!("Running update done");
    }

    fn build_net_inputs(&self, s: &GameState, a: Action) -> ArrayD<f32> {
        let mut v: Vec<_> = s.arr.into_iter().map(|n| *n as f32).collect();
        v.extend(a.vec());
        Array::from_shape_vec(IxDyn(&[1, 13]), v).unwrap()
    }
    fn generate_q(&self, sys: &SpatiumSys, s: &GameState, a: Action) -> f32 {
        let x = self.build_net_inputs(s, a);
        let p = self.run_network(sys, x);
        p[[0, 0]]
    }
    fn generate_q_val(&self, sys: &SpatiumSys, s: &GameState) -> ArrayD<f32> {
        let q_val: Vec<_> = Action::all()
            .into_iter()
            .map(|a| self.generate_q(sys, s, a))
            .collect();
        Array::from_shape_vec(IxDyn(&[1, 4]), q_val).unwrap()
    }
}

impl Network for SingleLayerNetwork {
    fn next_action(&mut self, sys: &SpatiumSys, rng: Option<RcRng>, s: &GameState) -> Action {
        let q_val = self.generate_q_val(sys, s);
        // sys.info(&format!("q_val={:?}", q_val));

        let dist = rand::distributions::Range::new(0., 0.3);

        let mut final_q_val = q_val;

        if let Some(mut rng) = rng {
            let noise = Array::from_shape_fn(IxDyn(&[1, 4]), |_| dist.ind_sample(&mut rng) as f32);
            // sys.info(&format!("noise={:?}", noise));
            final_q_val = final_q_val + noise;
        }

        // sys.info(&format!("final_q_val={:?}", final_q_val));
        use std::f32::MIN;
        let mut max_i = 0;
        let mut max_q = MIN;
        for (i, q) in final_q_val.iter().enumerate() {
            if *q > max_q {
                max_i = i;
                max_q = *q;
            }
        }

        // println!("max_i={:?}", max_i);

        // sys.info(&format!("{:?}", p));

        // let noise: ArrayD<f32> = ArrayD::random_using(IxDyn(&[1, 4]), Range::new(0., 3.), &mut rng);
        // println!("noise={:?}", noise);

        // let n = p + noise;
        // println!("n={:?}", n);

        // Action::Down
        // (p[[0, 0]] as usize).into()
        max_i.into()
    }
    fn result(
        &mut self,
        sys: &SpatiumSys,
        s: GameState,
        a: &Action,
        _s1: &GameState,
        r: usize,
        done: bool,
    ) {
        let r = r as f32;
        self.episode.push((s, a.to_owned(), r));

        if done {
            if r == 0. {
                return;
            }

            // println!("{:?}", self.episode);
            // update weights
            for (i, &mut (_, _, ref mut er)) in self.episode.iter_mut().rev().enumerate() {
                *er = (0. as f32).max(r - (i as f32 * 1.)) / 10.0;
            }
            // println!("{:?}", self.episode);
            let x: Vec<f32> = (&self.episode)
                .iter()
                .filter(|&&(_, _, ref er)| *er > 0.)
                .map(|&(ref es, ref ea, _)| self.build_net_inputs(&es, *ea))
                .fold(vec![], |mut n, a| {
                    n.extend(a.iter());
                    n
                });
            let y: Vec<_> = (&self.episode)
                .iter()
                .filter(|&&(_, _, ref er)| *er > 0.)
                .map(|&(_, _, ref er)| *er)
                .collect();
            if y.is_empty() {
                return;
            }

            let x = Array::from_shape_vec(IxDyn(&[y.len(), 13]), x).unwrap();
            let y = Array::from_shape_vec(IxDyn(&[y.len(), 1]), y).unwrap();

            // println!("x={:?}", x);
            // println!("y={:?}", y);

            self.run_update(x, y);
            // let inputs = self.episode.into_iter().map(|(s, a, r)| a.into());

            macro_rules! get_act_val {
                ( $ix:expr ) => {{
                    let mut s = GameState { arr: Array::zeros((3, 3)) };
                    s.arr[$ix] = 1;
                    let q_val = self.generate_q_val(sys, &s);
                    let (i, v) = argmax(q_val.iter());
                    let i: Action = i.into();
                    (i, v)
                }}
            }
            println!("Right {:?}", get_act_val!([0, 0]));
            println!("Right {:?}", get_act_val!([0, 1]));
            println!("Down {:?}", get_act_val!([0, 2]));
            println!("Down {:?}\n", get_act_val!([1, 2]));

            println!("Down {:?}", get_act_val!([0, 0]));
            println!("Down {:?}", get_act_val!([1, 0]));
            println!("Right {:?}", get_act_val!([2, 0]));
            println!("Right {:?}", get_act_val!([2, 1]));
        }
    }
}

fn argmax<'a, I: Iterator<Item = &'a f32>>(arr: I) -> (usize, f32) {
    use std;
    arr.enumerate()
        .fold((0, std::f32::MIN), |(max_i, max_n), (i, n)| {
            if *n > max_n {
                (i, *n)
            } else {
                (max_i, max_n)
            }
        })
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
