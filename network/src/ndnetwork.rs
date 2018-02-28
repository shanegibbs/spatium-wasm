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

fn neuron_sum_input(input: &Matrix, weights: &Matrix, bias: &Matrix) -> Matrix {
    assert_eq!(weights.cols(), bias.cols());
    assert_eq!(weights.rows(), input.cols());
    input.dot(weights) + bias
}

fn neuron_output(input: &Matrix, weights: &Matrix, bias: &Matrix) -> Matrix {
    Sigmoid::eval(neuron_sum_input(input, weights, bias))
}

fn update_hidden_layer_weights(
    layer_input: &Matrix,
    forward_weights_grade: &Matrix,
    weights: &Matrix,
    bias: &Matrix,
    lr: f64,
) -> (Matrix, Matrix, Matrix) {
    println!("\n hidden_layer_input={:?}", layer_input);
    // println!("\n weights={:?}", weights);
    // println!("\n bias={:?}", bias);
    println!("\n forward_weights_grade={:?}", forward_weights_grade);
    let inputsum = neuron_sum_input(layer_input, weights, bias);
    println!("\n inputsum={:?}", inputsum);

    let output = Sigmoid::eval(inputsum.clone());
    println!("\n output={:?}", output);

    let grad_output_to_inputsum = Sigmoid::derivative(inputsum.clone());
    println!("\n grad_output_to_inputsum={:?}", grad_output_to_inputsum);

    // TODO: calculate grad to output. Is sum of weights grad to each nuron.
    // in this case grad to output is just forward_weights_grade

    // TODO: grad_e_to_inputsum is wrong
    let grad_e_to_inputsum = forward_weights_grade.clone() * grad_output_to_inputsum;
    println!("\n grad_e_to_inputsum={:?}", grad_e_to_inputsum);

    let grade_e_to_weight = layer_input.t().clone().dot(&grad_e_to_inputsum);
    // println!("\n grade_e_to_weight={:?}", grade_e_to_weight);

    let change = grade_e_to_weight.clone() * -lr; // * -1 here to minimize error
                                                  // println!("\n change={:?}", change);
    let new_weights = weights.clone() + change;

    let change_b = grad_e_to_inputsum * -lr;
    // println!("\n change_b={:?}", change_b);
    let new_bias = bias.clone() + change_b;

    (new_weights, new_bias, grade_e_to_weight.t().to_owned())
}

fn update_output_layer_weights(
    layer_input: &Matrix,
    ideal_output: &Matrix,
    weights: &Matrix,
    bias: &Matrix,
    lr: f64,
) -> (Matrix, Matrix, Matrix) {
    // println!("\n layer_input={:?}", layer_input);
    // println!("\n weights={:?}", weights);

    let inputsum = neuron_sum_input(layer_input, weights, bias);
    // println!("\n inputsum={:?}", inputsum);

    let output = Sigmoid::eval(inputsum.clone());
    // println!("\n output={:?}", output);
    assert_eq!(output.dim(), ideal_output.dim());

    let grad_e_to_output = (output - ideal_output) * 2.0;
    // println!("\n grad_e_to_output={:?}", grad_e_to_output);
    let grad_output_to_inputsum = Sigmoid::derivative(inputsum.clone());
    // println!("\n grad_output_to_input={:?}", grad_output_to_inputsum);

    let grad_e_to_inputsum = grad_e_to_output * grad_output_to_inputsum;
    // println!("\n grad_e_to_input={:?}", grad_e_to_inputsum);

    let grade_e_to_weight = layer_input.t().clone().dot(&grad_e_to_inputsum);
    // println!("\n grade_e_to_weight={:?}", grade_e_to_weight);

    let change = grade_e_to_weight.clone() * -lr; // * -1 here to minimize error
    let new_weights = weights.clone() + change;

    let change_b = grad_e_to_inputsum * -lr;
    // println!("\n change_b={:?}", change_b);
    let new_bias = bias.clone() + change_b;

    (new_weights, new_bias, grade_e_to_weight.t().to_owned())
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

        let result = update_output_layer_weights(&input, &ideal_output, &weights, &bias, 0.1);
        // println!("\n{:?}", result);
        assert_eq!(
            (result.0, result.1),
            (
                arr2(&[[0.5017796123978443], [0.6035592247956885]]),
                arr2(&[[0.8088980619892214]])
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

        let result = update_output_layer_weights(&input, &ideal_output, &weights, &bias, 0.1);
        // println!("\n{:?}", result);
        assert_eq!(
            (result.0, result.1),
            (
                arr2(&[
                    [0.5017796123978443, 0.6940762519421037],
                    [0.6035592247956885, 0.7881525038842075],
                ]),
                arr2(&[[0.8088980619892214, 0.1703812597105187]])
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

        let result = update_output_layer_weights(&input, &ideal_output, &weights, &bias, 0.1);
        // println!("\n{:?}", result);
        assert_eq!(
            (result.0, result.1),
            (
                arr2(&[
                    [0.5017796123978443, 0.6940762519421037, 0.09424635425260525],
                    [0.6035592247956885, 0.7881525038842075, 0.1884927085052105]
                ]),
                arr2(&[[0.8088980619892214, 0.1703812597105187, 0.2712317712630262]])
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
            let (weights_new, bias_new, _) =
                update_output_layer_weights(&input, &ideal_output, &weights, &bias, 0.1);
            weights = weights_new;
            bias = bias_new;

            err = mse(&neuron_output(&input, &weights, &bias), &ideal_output);
            if i == 0 {
                println!("mse={:?}", err);
            }
        }

        println!("mse={:?}", err);
        assert!(err < 0.00120);
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

        let mut weights = arr2(&[[0.3], [0.7]]);
        let mut bias = arr2(&[[0.5]]);

        let mut x = vec![];
        let mut y_err = vec![];
        let mut w1 = vec![];
        let mut w2 = vec![];
        let mut b1 = vec![];

        for i in 0..500 {
            for t in &training_set {
                let (weights_new, bias_new, _) =
                    update_output_layer_weights(&t.0, &t.1, &weights, &bias, 0.1);
                weights = weights_new;
                bias = bias_new;
            }

            let mut err = 0.0;
            for t in &training_set {
                err += mse(&neuron_output(&t.0, &weights, &bias), &t.1);
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
            assert_eq!(t.1[(0, 0)], answer.round());
        }
    }

    #[test]
    fn test_update_output_weights_bitxor_training() {
        let training_set = [
            (arr2(&[[0.0, 0.0]]), arr2(&[[0.0]])),
            (arr2(&[[0.0, 1.0]]), arr2(&[[1.0]])),
            (arr2(&[[1.0, 0.0]]), arr2(&[[1.0]])),
            (arr2(&[[1.0, 1.0]]), arr2(&[[0.0]])),
        ];
        let inputs: Vec<_> = training_set.iter().map(|v| &v.0).collect();
        let ideal_outputs: Vec<_> = training_set.iter().map(|v| &v.1).collect();

        let lr = 0.1;
        let mut weights_1 = arr2(&[[0.3, 0.1], [0.7, 0.2]]);
        let mut bias_1 = arr2(&[[0.2, 0.6]]);
        let mut weights_2 = arr2(&[[0.5], [0.7]]);
        let mut bias_2 = arr2(&[[0.5]]);

        for i in 0..1000 {
            // for t in &training_set {
            // let input = &t.0;
            // let ideal_output = &t.1;
            let input = &training_set[3].0;
            let ideal_output = &training_set[3].1;

            println!("\n input={:?}", input);
            println!("\n ideal_output={:?}", ideal_output);

            let layer_1_output = neuron_output(input, &weights_1, &bias_1);
            let layer_2_output = neuron_output(&layer_1_output, &weights_2, &bias_2);
            println!("\n layer_1_output={:?}", layer_1_output);
            println!("\n layer_2_output={:?}", layer_2_output);

            let err_before = mse(&layer_2_output, ideal_output);

            let (weights_2_new, bias_2_new, grade_e_to_weight) =
                update_output_layer_weights(&layer_1_output, ideal_output, &weights_2, &bias_2, lr);

            println!("\n weights_2_new={:?}", weights_2_new);
            println!("\n bias_2_new={:?}", bias_2_new);

            let (weights_1_new, bias_1_new, _) =
                update_hidden_layer_weights(&input, &grade_e_to_weight, &weights_1, &bias_1, lr);

            println!("\n weights_1_new={:?}", weights_1_new);
            println!("\n bias_1_new={:?}", bias_1_new);

            weights_1 = weights_1_new;
            bias_1 = bias_1_new;

            weights_2 = weights_2_new;
            bias_2 = bias_2_new;

            let layer_1_output = neuron_output(input, &weights_1, &bias_1);
            let layer_2_output = neuron_output(&layer_1_output, &weights_2, &bias_2);
            let err_after = mse(&layer_2_output, ideal_output);
            let improvement = err_before - err_after;

            println!("\n err_before={:?}", err_before);
            println!("\n err_after={:?}", err_after);
            println!("\n improvement={:?}", improvement);
            assert!(improvement > 0.0);
            panic!();
            // }

            let mut err = 0.0;
            for t in &training_set {
                err += mse(
                    &neuron_output(
                        &neuron_output(&t.0, &weights_1, &bias_1),
                        &weights_2,
                        &bias_2,
                    ),
                    &t.1,
                );
            }
            err /= training_set.len() as f64;
            println!("\n err={:?}", err);
        }

        println!("\n weights_1={:?}", weights_1);
        println!("\n bias_1={:?}", bias_1);
        println!("\n weights_2={:?}", weights_2);
        println!("\n bias_2={:?}", bias_2);

        for t in &training_set {
            let answer = neuron_output(
                &neuron_output(&t.0, &weights_1, &bias_1),
                &weights_2,
                &bias_2,
            )[(0, 0)];
            println!("\nanswer={:?}", answer);
            // assert_eq!(t.1[(0, 0)], answer.round());
        }
    }
}
