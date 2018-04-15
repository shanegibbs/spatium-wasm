use super::*;

use std::collections::HashMap;

use ndarray_rand::RandomExt;
use ndarray::{Array, Ix1};
use ndarray::prelude::*;
use rand::distributions::Range;
use rng::RcRng;

pub struct QTable {
    q: HashMap<ArrayD<u8>, Array<f32, Ix1>>,
}

impl QTable {
    pub fn new() -> Self {
        QTable { q: HashMap::new() }
    }
}

impl Network for QTable {
    fn next_action(&mut self, _: &SpatiumSys, rng: Option<RcRng>, s: &GameState) -> (Action, f32) {
        let q_val = self.q
            .get(&s.arr)
            .map(|a| a.to_owned())
            .unwrap_or(Array::zeros(4));

        let mut final_q_val = q_val;

        if let Some(mut rng) = rng {
            let noise: Array1<f32> = Array1::random_using(4, Range::new(0., 3.), &mut rng);
            // self.sys.info(format!("{}", noise));
            final_q_val = final_q_val + noise;
        }

        let (action_i, maxq) = argmax(&final_q_val);
        (action_i.into(), maxq)
    }
    fn result(
        &mut self,
        _sys: &SpatiumSys,
        _rng: RcRng,
        s: GameState,
        a: &Action,
        s1: &GameState,
        r: usize,
        _done: bool,
    ) -> Metrics {
        let mut q_val = self.q
            .get(&s.arr)
            .map(|a| a.to_owned())
            .unwrap_or(Array::zeros(4));

        // update Q
        let s1_q_val = self.q
            .get(&s1.arr)
            .map(|a| a.to_owned())
            .unwrap_or(Array::zeros(4));
        let r1 = argmax(&s1_q_val).1;

        let lr = 0.8f32;
        let y = 0.95f32;

        let action_i = a.into();
        let existing = q_val[[action_i]];
        q_val[[action_i]] = existing + lr * (r as f32 + y * r1 - existing);
        self.q.insert(s.arr, q_val);

        Default::default()
    }
}

fn argmax(arr: &Array1<f32>) -> (usize, f32) {
    use std;
    arr.iter()
        .enumerate()
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

    #[test]
    fn test_argmax() {
        let mut n: Array<f32, Ix1> = Array::zeros(4);
        n[[0]] = 0f32;
        n[[1]] = 1f32;
        n[[2]] = 5f32;
        n[[3]] = 2f32;
        assert_eq!(argmax(&n), (2, 5f32));

        let mut n: Array<f32, Ix1> = Array::zeros(4);
        n[[0]] = -5f32;
        n[[1]] = -6f32;
        n[[2]] = -7f32;
        n[[3]] = -2f32;
        assert_eq!(argmax(&n), (3, -2f32));
    }

}
