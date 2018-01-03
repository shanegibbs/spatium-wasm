use ndarray;
use ndarray::prelude::*;
use std::cmp::max;

type Matrix = Array2<f64>;

fn calculate_layer() {}

pub trait ActivationFn {
    fn eval(Matrix) -> Matrix;
    fn derivative(Matrix) -> Matrix;
}

struct Sigmoid;
impl ActivationFn for Sigmoid {
    fn eval(n: Matrix) -> Matrix {
        1f64 / (1f64 + (-n).mapv(f64::exp))
    }
    fn derivative(n: Matrix) -> Matrix {
        let sig = Sigmoid::eval(n);
        sig.clone() * (1f64 - &sig)
    }
}

struct Relu;
impl ActivationFn for Relu {
    fn eval(n: Matrix) -> Matrix {
        n.mapv(|n| 0f64.max(n))
    }
    fn derivative(n: Matrix) -> Matrix {
        n
    }
}

struct Softmax;
impl ActivationFn for Softmax {
    fn eval(n: Matrix) -> Matrix {
        // println!("softmax_in=\n{:?}", n);
        let n = n.mapv(f64::exp);
        // println!("softmax_to_e=\n{:?}", n);
        let s = n.scalar_sum();
        // println!("softmax_sum={}", s);
        n.mapv(|n| n / s)
    }
    fn derivative(n: Matrix) -> Matrix {
        n
    }
}

fn mse(output: &Matrix, ideal_output: &Matrix) -> f64 {
    assert_eq!(output.cols(), ideal_output.cols());
    let sum = (ideal_output - output).mapv(|n| n.powi(2)).scalar_sum();
    sum / output.cols() as f64
}

// https://github.com/mnielsen/neural-networks-and-deep-learning/tree/master/src

fn update_output_weights(
    layer_input: &Matrix,
    ideal_output: &Matrix,
    weights: &Matrix,
    bias: &Matrix,
    lr: f64,
) -> (Matrix, Matrix) {
    // println!("\n layer_input={:?}", layer_input);
    // println!("\n weights={:?}", weights);

    assert_eq!(weights.cols(), bias.cols());
    assert_eq!(weights.rows(), layer_input.cols());
    let input = layer_input.dot(weights) + bias;
    // println!("\n input={:?}", input);

    let output = Sigmoid::eval(input.clone());
    // println!("\n output={:?}", output);
    assert_eq!(output.dim(), ideal_output.dim());

    let grad_e_to_output = output - ideal_output;
    // println!("\n grad_e_to_output={:?}", grad_e_to_output);
    let grad_output_to_input = Sigmoid::derivative(input.clone());
    // println!("\n grad_output_to_input={:?}", grad_output_to_input);
    let grad_input_to_weight = layer_input.clone();
    // println!("\n grad_input_to_weight={:?}", grad_input_to_weight);

    let grad_e_to_input = grad_e_to_output * grad_output_to_input;
    // println!("\n grad_e_to_input={:?}", grad_e_to_input);

    let grade_e_to_weight = layer_input.t().clone().dot(&grad_e_to_input);
    // println!("\n grade_e_to_weight={:?}", grade_e_to_weight);

    let change = grade_e_to_weight * -lr; // * -1 here to minimize error
    let new_weights = weights.clone() + change;

    let change_b = grad_e_to_input * -lr;
    // println!("\n change_b={:?}", change_b);
    let new_bias = bias.clone() + change_b;

    (new_weights, new_bias)
}

#[cfg(test)]
mod tests {
    extern crate gnuplot;
    extern crate test;

    use self::gnuplot::{AxesCommon, Caption, Color, Figure};
    use super::*;

    #[test]
    fn test_update_output_weights_single_neuron() {
        let input = arr2(&[[0.2, 0.4]]);
        // println!("\n{:?}", input);

        let ideal_output = arr2(&[[1.0]]);
        let weights = arr2(&[[0.5], [0.6]]);
        let bias = arr2(&[[0.8]]);

        let result = update_output_weights(&input, &ideal_output, &weights, &bias, 0.1);
        // println!("\n{:?}", result);
        assert_eq!(
            result,
            (
                arr2(&[[0.5008898061989221], [0.6017796123978443]]),
                arr2(&[[0.8044490309946107]])
            )
        );
    }

    #[test]
    fn test_update_output_weights_two_neuron() {
        let input = arr2(&[[0.2, 0.4]]);
        // println!("\n{:?}", input);

        let ideal_output = arr2(&[[1.0, 0.0]]);
        let weights = arr2(&[[0.5, 0.7], [0.6, 0.8]]);
        let bias = arr2(&[[0.8, 0.2]]);

        let result = update_output_weights(&input, &ideal_output, &weights, &bias, 0.1);
        // println!("\n{:?}", result);
        assert_eq!(
            result,
            (
                arr2(&[
                    [0.5008898061989221, 0.6970381259710519],
                    [0.6017796123978443, 0.7940762519421037]
                ]),
                arr2(&[[0.8044490309946107, 0.18519062985525936]])
            )
        );
    }

    #[test]
    fn test_update_output_weights_three_neuron() {
        // 2 inputs, 3 outputs/neurons
        let input = arr2(&[[0.2, 0.4]]);
        // println!("\n{:?}", input);

        let ideal_output = arr2(&[[1.0, 0.0, 0.0]]);
        let weights = arr2(&[[0.5, 0.7, 0.1], [0.6, 0.8, 0.2]]);
        let bias = arr2(&[[0.8, 0.2, 0.3]]);

        let result = update_output_weights(&input, &ideal_output, &weights, &bias, 0.1);
        // println!("\n{:?}", result);
        assert_eq!(
            result,
            (
                arr2(&[
                    [0.5008898061989221, 0.6970381259710519, 0.09712317712630263],
                    [0.6017796123978443, 0.7940762519421037, 0.19424635425260525]
                ]),
                arr2(&[
                    [0.8044490309946107, 0.18519062985525936, 0.2856158856315131]
                ])
            )
        );
    }

    #[test]
    fn test_update_output_weights_minimize() {
        // 2 inputs, 3 outputs/neurons
        let input = arr2(&[[0.2, 0.4]]);
        // println!("\n{:?}", input);
        let ideal_output = arr2(&[[1.0, 0.0, 0.0]]);

        let mut err = 0.0;
        let mut weights = arr2(&[[0.5, 0.7, 0.1], [0.6, 0.8, 0.2]]);
        let mut bias = arr2(&[[0.8, 0.2, 0.3]]);

        for i in 0..2000 {
            let (weights_new, bias_new) =
                update_output_weights(&input, &ideal_output, &weights, &bias, 0.1);
            weights = weights_new;
            bias = bias_new;

            err = mse(&Sigmoid::eval(input.dot(&weights) + &bias), &ideal_output);
            if i == 0 {
                println!("mse={:?}", err);
            }
        }

        println!("mse={:?}", err);
        assert!(err < 0.0026);
    }

    #[test]
    fn test_update_output_weights_bitand_training() {
        let training_set = [
            (arr2(&[[0.0, 0.0]]), arr2(&[[0.0]])),
            (arr2(&[[0.0, 1.0]]), arr2(&[[0.0]])),
            (arr2(&[[1.0, 0.0]]), arr2(&[[0.0]])),
            (arr2(&[[1.0, 1.0]]), arr2(&[[1.0]])),
        ];
        let inputs: Vec<_> = training_set.iter().map(|v| &v.0).collect();
        let ideal_outputs: Vec<_> = training_set.iter().map(|v| &v.1).collect();
        // println!("\ninputs={:?}", inputs);
        // println!("\noutputs={:?}", ideal_outputs);

        let mut weights = arr2(&[[0.3], [0.7]]);
        let mut bias = arr2(&[[0.5]]);

        let mut x = vec![];
        let mut y_err = vec![];
        let mut w1 = vec![];
        let mut w2 = vec![];
        let mut b1 = vec![];

        for i in 0..500 {
            for t in &training_set {
                let (weights_new, bias_new) =
                    update_output_weights(&t.0, &t.1, &weights, &bias, 0.1);
                weights = weights_new;
                bias = bias_new;
            }

            let mut err = 0.0;
            for t in &training_set {
                err += mse(&Sigmoid::eval(t.0.dot(&weights) + &bias), &t.1);
            }
            err /= training_set.len() as f64;

            x.push(i);
            y_err.push(err);
            w1.push(weights[(0, 0)]);
            w2.push(weights[(1, 0)]);
            b1.push(bias[(0, 0)]);
            // println!("err={:?}", err);
        }

        println!("\n weights={:?}", weights);
        println!("\n bias={:?}", bias);

        let mut fg = Figure::new();
        fg.set_terminal("png size 1200, 400", "test-bitand.png");
        {
            let axes2d = fg.axes2d().set_pos_grid(1, 2, 0).set_title("Error", &[]);
            axes2d.lines(&x, &y_err, &[Caption("Error"), Color("red")]);
        }
        {
            let axes2d = fg.axes2d().set_pos_grid(1, 2, 1).set_title("Weights", &[]);
            axes2d.lines(&x, &w1, &[Caption("w1"), Color("blue")]);
            axes2d.lines(&x, &w2, &[Caption("w2"), Color("orange")]);
            axes2d.lines(&x, &b1, &[Caption("b1"), Color("green")]);
        }
        fg.show();

        for t in &training_set {
            let answer = Sigmoid::eval(t.0.dot(&weights) + &bias)[(0, 0)];
            println!("\nanswer={:?}", answer);
            let answer = if answer >= 0.5 { 1.0 } else { 0.0 };
            assert_eq!(t.1[(0, 0)], answer);
        }
    }

    fn test_layer() {
        let lr = 0.1f64;

        // input

        let input = arr2(&[[0.1, 0.2, 0.7]]);
        println!("\ninput=\n{:?}", input);

        // ideal output

        let ideal_output = arr2(&[[1.0, 0.0, 0.0]]);
        println!("\nideal_output=\n{:?}", ideal_output);

        // model

        let weights_ij = arr2(&[[0.1, 0.2, 0.3], [0.3, 0.2, 0.7], [0.4, 0.3, 0.9]]);
        let bias_ij = Array::from_elem((1, 3), 1.);
        println!("\nweights_ij=\n{:?}", weights_ij);
        println!("\nbias_ij=\n{:?}", bias_ij);

        let weights_jk = arr2(&[[0.2, 0.3, 0.5], [0.3, 0.5, 0.7], [0.6, 0.4, 0.8]]);
        let bias_jk = Array::from_elem((1, 3), 1.);
        println!("\nweights_jk=\n{:?}", weights_jk);
        println!("\nbias_jk=\n{:?}", bias_jk);

        let weights_kl = arr2(&[[0.1, 0.4, 0.8], [0.3, 0.7, 0.2], [0.5, 0.2, 0.9]]);
        let bias_kl = Array::from_elem((1, 3), 1.);
        println!("\nweights_kl=\n{:?}", weights_kl);
        println!("\nbias_kl=\n{:?}", bias_kl);

        // forward

        let h1_out = Sigmoid::eval(input.dot(&weights_ij) + &bias_ij);
        println!("\nh1_out=\n{:?}", h1_out);

        let h2_out = Sigmoid::eval(h1_out.dot(&weights_jk) + &bias_jk);
        println!("\nh2_out=\n{:?}", h2_out);

        let output = Sigmoid::eval(h2_out.dot(&weights_kl) + &bias_kl);
        println!("\nout_out=\n{:?}", output);

        // error

        let mse_val = mse(&output, &ideal_output);
        println!("\nmse=\n{:?}", mse_val); // 0.330125

        // update kl weights

        let wrt_output = ideal_output.clone() - &output;
        println!("\nwrt_output=\n{:?}", wrt_output);

        let layer_out = h2_out.dot(&weights_kl) + &bias_kl;
        let wrt_activation = Sigmoid::derivative(layer_out.clone());
        println!("\nwrt_activation=\n{:?}", wrt_activation);

        let wrt_weight = h2_out;
        println!("\nwrt_weight=\n{:?}", wrt_weight);

        let n = wrt_output.clone() * wrt_activation;
        let layer_error = output.t().dot(&n);
        println!("\nlayer_error=\n{:?}", layer_error);

        let change = layer_error.clone() * lr;
        println!("\nchange=\n{:?}", change);

        let weights_kl_updated = weights_kl.clone() + change;
        println!("\nweights_kl_updated=\n{:?}", weights_kl_updated);

        // update jk weights

        let wrt_output = weights_kl.clone() * &layer_error;
        println!("\nwrt_output=\n{:?}", wrt_output);

        let wrt_activation = Sigmoid::derivative(h1_out.dot(&weights_jk) + &bias_jk);
        println!("\nwrt_activation=\n{:?}", wrt_activation);

        let wrt_weight = h1_out;
        println!("\nwrt_weight=\n{:?}", wrt_weight);

        let layer_error = wrt_output * wrt_activation * wrt_weight;
        println!("\nlayer_error=\n{:?}", layer_error);

        let change = layer_error.clone() * lr;
        println!("\nchange=\n{:?}", change);

        let weights_jk_updated = weights_jk.clone() + change;
        println!("\nweights_jk_updated=\n{:?}", weights_jk_updated);

        // update ik weights

        let wrt_output = weights_ij.clone() * &layer_error;
        println!("\nwrt_output=\n{:?}", wrt_output);

        let wrt_activation = Sigmoid::derivative(input.dot(&weights_ij) + &bias_ij);
        println!("\nwrt_activation=\n{:?}", wrt_activation);

        let wrt_weight = input.clone();
        println!("\nwrt_weight=\n{:?}", wrt_weight);

        let layer_error = wrt_output * wrt_activation * wrt_weight;
        println!("\nlayer_error=\n{:?}", layer_error);

        let change = layer_error.clone() * lr;
        println!("\nchange=\n{:?}", change);

        let weights_ij_updated = weights_ij.clone() + change;
        println!("\nweights_ij_updated=\n{:?}", weights_ij_updated);


        let h1_out = Sigmoid::eval(input.dot(&weights_ij) + &bias_ij);
        let h2_out = Sigmoid::eval(h1_out.dot(&weights_jk_updated) + &bias_jk);
        let output = Sigmoid::eval(h2_out.dot(&weights_kl_updated) + &bias_kl);
        let mse_val = mse(&output, &ideal_output);
        println!("\nmse=\n{:?}", mse_val); // 0.330125
    }
}
