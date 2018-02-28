use game::GameState;
use action::*;

use std::collections::HashMap;
use ndarray_rand::RandomExt;
use ndarray::{Array, Ix1, Ix2};
use ndarray::prelude::*;

use pcg_rand::Pcg32Basic;

use rand::distributions::Range;

pub trait Network {
    fn next_action(&mut self, &mut Pcg32Basic, &GameState) -> Action;
    fn result(&mut self, GameState, &Action, &GameState, usize, bool);
}

pub struct QTable {
    q: HashMap<Array<u8, Ix2>, Array<f32, Ix1>>,
}

impl QTable {
    pub fn new() -> Self {
        QTable { q: HashMap::new() }
    }
}

impl Network for QTable {
    fn next_action(&mut self, rng: &mut Pcg32Basic, s: &GameState) -> Action {
        let q_val = self.q
            .get(&s.arr)
            .map(|a| a.to_owned())
            .unwrap_or(Array::zeros((4)));

        let noise: Array1<f32> = Array1::random_using((4), Range::new(0., 3.), rng);
        // self.sys.info(format!("{}", noise));

        let final_q_val = q_val.clone() + noise;

        let action_i = argmax(&final_q_val).0;
        action_i.into()
    }
    fn result(&mut self, s: GameState, a: &Action, s1: &GameState, r: usize, _done: bool) {
        let mut q_val = self.q
            .get(&s.arr)
            .map(|a| a.to_owned())
            .unwrap_or(Array::zeros((4)));

        // update Q
        let s1_q_val = self.q
            .get(&s1.arr)
            .map(|a| a.to_owned())
            .unwrap_or(Array::zeros((4)));
        let r1 = argmax(&s1_q_val).1;

        let lr = 0.8f32;
        let y = 0.95f32;

        let action_i = a.into();
        let existing = q_val[[action_i]];
        q_val[[action_i]] = existing + lr * (r as f32 + y * r1 - existing);
        self.q.insert(s.arr, q_val);
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

mod test {

    #[test]
    fn test_argmax() {
        let mut n: Array<f32, Ix1> = Array::zeros((4));
        n[[0]] = 0f32;
        n[[1]] = 1f32;
        n[[2]] = 5f32;
        n[[3]] = 2f32;
        assert_eq!(argmax(&n), (2, 5f32));

        let mut n: Array<f32, Ix1> = Array::zeros((4));
        n[[0]] = -5f32;
        n[[1]] = -6f32;
        n[[2]] = -7f32;
        n[[3]] = -2f32;
        assert_eq!(argmax(&n), (3, -2f32));
    }

}
