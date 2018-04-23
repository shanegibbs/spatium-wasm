use ag::{self, Tensor};
use ndarray::prelude::*;
use RcRng;
use ag::ops::gradient_descent_ops::*;

#[derive(Clone, Debug)]
pub struct Weights {
    pub lr: f32,
    inputs: isize,
    outputs: isize,
    pub w: ArrayD<f32>,
    pub b: ArrayD<f32>,
    pub hw: ArrayD<f32>,
    pub hb: ArrayD<f32>,
}

impl Weights {
    pub fn new(inputs: usize, outputs: usize, hidden: usize, lr: f32, rng: RcRng) -> Self {
        let arr_rng = ag::ndarray_ext::ArrRng::new(rng);

        let mut w = arr_rng.glorot_uniform(&[inputs, hidden]);
        w.mapv_inplace(|n| n.abs());
        let b = ag::ndarray_ext::zeros(&[1, hidden]);

        let mut hw = arr_rng.glorot_uniform(&[hidden, outputs]);
        hw.mapv_inplace(|n| n.abs());
        let hb = ag::ndarray_ext::zeros(&[1, outputs]);

        Weights {
            lr,
            inputs: inputs as isize,
            outputs: outputs as isize,
            w,
            b,
            hw,
            hb,
        }
    }
}

impl<'w> From<&'w Weights> for NeuralNet {
    fn from(weights: &'w Weights) -> Self {
        let x = ag::placeholder(&[-1, weights.inputs]);
        let y = ag::placeholder(&[-1, weights.outputs]);

        let w1 = ag::variable(weights.w.clone());
        let b1 = ag::variable(weights.b.clone());
        let w2 = ag::variable(weights.hw.clone());
        let b2 = ag::variable(weights.hb.clone());

        let z1 = ag::matmul(&x, &w1) + &b1;
        let zz1 = ag::relu(&z1);

        let z2 = ag::matmul(&zz1, &w2) + &b2;
        let zz2 = ag::relu(&z2);

        let e = ag::sub(&y, &zz2);
        let se = ag::square(&e);
        let mse_each = ag::reduce_sum(&se, &[1], false);
        let mse = ag::reduce_sum(&se, &[0, 1], false);

        let params = [&w1.clone(), &b1.clone(), &w2.clone(), &b2.clone()];
        let grads = ag::grad(&[&mse], &params);

        // let mut adam = Adam {
        //     alpha: weights.lr,
        //     eps: 1e-08,
        //     b1: 0.9,
        //     b2: 0.999,
        //     stateful_params: BTreeMap::new(),
        // };
        // let mut update_ops = adam.compute_updates(&params, &grads);
        let mut sgd = sgd::SGD { lr: weights.lr };
        let mut update_ops = sgd.compute_updates(&params, &grads);

        update_ops.insert(0, mse.clone());

        let max = ag::reduce_max(&zz2, &[1], false);
        let a = ag::argmax(&zz2, 1, false);

        NeuralNet {
            lr: weights.lr,
            inputs: weights.inputs as usize,
            outputs: weights.outputs as usize,
            x: x,
            y: y,
            w1,
            b1,
            w2,
            b2,
            update_ops: update_ops,
            action: a,
            q_values: zz2,
            q_value_max: max,
            mse_each: mse_each,
            _misc: vec![],
        }
    }
}

pub struct NeuralNet {
    lr: f32,
    inputs: usize,
    outputs: usize,
    x: Tensor,
    y: Tensor,
    w1: Tensor,
    b1: Tensor,
    w2: Tensor,
    b2: Tensor,
    update_ops: Vec<Tensor>,
    action: Tensor,
    q_values: Tensor,
    q_value_max: Tensor,
    _misc: Vec<Tensor>,
    mse_each: Tensor,
}

impl NeuralNet {
    pub fn run(&self, x_val: Array2<f32>) -> (Array1<f32>, Array2<f32>, Array1<f32>) {
        let len = x_val.shape()[0];

        let x_val = x_val.into_dyn();
        let result = ag::eval(
            &[&self.action, &self.q_values, &self.q_value_max],
            &[(&self.x, &x_val)],
        );

        let a_val = result[0].clone().expect("eval a_val");
        let q_val = result[1].clone().expect("eval q_val");
        let max_q = result[2].clone().expect("eval max_q");

        return (
            a_val.into_shape(len).expect("a_val shape"),
            q_val.into_shape((len, self.outputs)).expect("q_val shape"),
            max_q.into_shape(len).expect("max_q shape"),
        );
    }
    pub fn update(&mut self, x_val: Array2<f32>, y_val: Array2<f32>) -> f32 {
        let x_val = x_val.into_dyn();
        let y_val = y_val.into_dyn();

        let feeds = &[(&self.x, &x_val), (&self.y, &y_val)];
        let result = ag::eval(&self.update_ops, feeds);

        let mse = result[0].as_ref().unwrap();
        assert_eq!(mse.shape(), [] as [usize; 0]);
        mse[[]]
    }
    pub fn _weights(&self) -> Vec<Vec<f32>> {
        let result = ag::eval(&[&self.w1, &self.b1, &self.w2, &self.b2], &[]);
        result
            .into_iter()
            .map(|r| r.unwrap().into_raw_vec())
            .collect()
    }
    pub fn mse_each(&self, x_val: Array2<f32>, y_val: Array2<f32>) -> ArrayD<f32> {
        let x_val = x_val.into_dyn();
        let y_val = y_val.into_dyn();
        let result = ag::eval(&[&self.mse_each], &[(&self.x, &x_val), (&self.y, &y_val)]);
        result.into_iter().next().unwrap().unwrap()
    }
    pub fn _misc(&self, x_val: Array2<f32>, y_val: Array2<f32>) {
        let x_val = x_val.into_dyn();
        let y_val = y_val.into_dyn();

        let feeds = &[(&self.x, &x_val), (&self.y, &y_val)];
        let result = ag::eval(&self._misc, feeds);
        let result: Vec<_> = result
            .into_iter()
            .map(|r| r.unwrap().into_raw_vec())
            .collect();

        println!("{:?}", result);
    }
    pub fn build_weights(&self) -> Weights {
        let new_vars = ag::eval(&[&self.w1, &self.b1, &self.w2, &self.b2], &[]);

        Weights {
            lr: self.lr,
            inputs: self.inputs as isize,
            outputs: self.outputs as isize,
            w: new_vars[0].clone().expect("run_update w"),
            b: new_vars[1].clone().expect("run_update b"),
            hw: new_vars[2].clone().expect("run_update hw"),
            hb: new_vars[3].clone().expect("run_update hb"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_update() {
        let rng = RcRng::new(Box::new(thread_rng()));

        let mut weights = Weights::new(2, 1, 4, 0.001, rng);
        let mut net: NeuralNet = (&weights).into();

        let x = Array::from_vec(vec![0., 0., 0., 1., 1., 0., 1., 1.])
            .into_shape((4, 2))
            .unwrap();
        let y = Array::from_vec(vec![0., 1., 1., 0.])
            .into_shape((4, 1))
            .unwrap();

        println!("x:\n{}", x);
        println!("y:\n{}", y);

        let w = net._weights();
        println!("w: {:?}", w);

        for _ in 0..1000 {
            net = (&weights).into();
            // let w = net.weights();
            let _e = net.update(x.clone(), y.clone());
            // println!("e: {}, w: {:?}", e, w);

            weights = net.build_weights();
            // net.misc(x.clone(), y.clone());
            net.mse_each(x.clone(), y.clone());
        }

        let w = net._weights();
        println!("w: {:?}", w);

        let result = net.run(x.clone());
        println!("p:\n{:?}", result.1);
    }
}
