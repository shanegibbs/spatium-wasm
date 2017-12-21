// #[derive(Debug, Clone)]
// struct Network {
//     layers: Vec<Layer>,
// }

#[derive(Debug, Clone)]
struct Layer {
    bias_weight: f64,
    neurons: Vec<Neuron>,
}

#[derive(Debug, Clone)]
struct Neuron {
    weights: Vec<f64>,
}

fn sigmoid_activation_fn(n: f64) -> f64 {
    1f64 / (1f64 + (-n).exp())
}

fn sigmoid_derivative_fn(n: f64) -> f64 {
    n * (1f64 - n)
}

impl Layer {
    fn forward(&self, input: &Vec<f64>) -> Vec<f64> {
        let mut output = vec![];
        for neuron in &self.neurons {
            let neuron_output = neuron.output(&input, self.bias_weight);
            let activation = sigmoid_activation_fn(neuron_output);
            output.push(activation);
        }
        output
    }
    fn backward_hidden_layer(&mut self,
                             input: &Vec<f64>,
                             forward: &Layer,
                             deltas: &Vec<f64>,
                             lr: f64) {
        // println!("Hidden layer");
        let output = self.forward(input);

        for (i, neuron) in self.neurons.iter_mut().enumerate() {

            let mut neuron_error = 0.0;
            for n in &forward.neurons {
                let in_neuron_w = n.weights[i];
                let in_neuron_delta = deltas[i];
                // println!("in_neuron_w={}", in_neuron_w);
                // println!("in_neuron_delta={}", in_neuron_delta);

                let in_neuron_err = in_neuron_w * in_neuron_delta;
                // println!("in_neuron_err={}", in_neuron_err);

                neuron_error += in_neuron_err;
            }
            // println!("hidden neuron_error={}", neuron_error);


            let neuron_output = output[i];
            // println!("neuron_output={}", neuron_output);

            let derivative = sigmoid_derivative_fn(neuron_output);
            // println!("neuron_derivative={}", derivative);

            let delta = neuron_error * derivative;
            // deltas.push(delta);
            // println!("delta={}", delta);

            let mut new_weights = vec![];
            for (j, w) in neuron.weights.iter().enumerate() {
                let input = input[j];
                let x = delta * input;
                // println!("x={}", x);
                new_weights.push(w - (lr * x));
            }

            neuron.weights = new_weights;
            // println!("new_weights={:?}", neuron.weights);
        }

    }
    fn backward_output_layer(&mut self, input: &Vec<f64>, ideal: &Vec<f64>, lr: f64) -> Vec<f64> {
        // used to calculate errors on next layer
        let mut deltas = vec![];

        let output = self.forward(input);

        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            // println!("\nNeuron {}", i);
            let neuron_output = output[i];
            // println!("neuron_output={}", neuron_output);

            let neuron_error = neuron_output - ideal[i];
            // println!("neuron_error={}", neuron_error);

            // angle of the curve (what direction we should change in)
            let derivative = sigmoid_derivative_fn(neuron_output);
            // println!("neuron_derivative={}", derivative);

            let delta = neuron_error * derivative;
            deltas.push(delta);
            // println!("delta={}", delta);

            let mut new_weights = vec![];
            for (j, w) in neuron.weights.iter().enumerate() {
                let input = input[j];
                let x = delta * input;
                // println!("x={}", x);
                new_weights.push(w - (lr * x));
            }

            neuron.weights = new_weights;
            // println!("new_weights={:?}", neuron.weights);
        }

        deltas
    }
}

impl Neuron {
    fn output(&self, input: &Vec<f64>, bias: f64) -> f64 {
        let mut output = bias;
        for (i, w) in input.iter().zip(&self.weights) {
            output += i * w;
        }
        output
    }
}

fn squared_error(ideal: f64, actual: f64) -> f64 {
    (ideal - actual).powi(2) / 2.
}

fn squared_error_acc(acc: f64, next: (&f64, &f64)) -> f64 {
    acc + squared_error(*next.0, *next.1)
}

fn squared_error_of_vec(ideal: &Vec<f64>, actual: &Vec<f64>) -> f64 {
    ideal.iter().zip(actual).fold(0., squared_error_acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward() {
        let mut hidden = Layer {
            bias_weight: 0.35,
            neurons: vec![Neuron { weights: vec![0.15, 0.2] },
                          Neuron { weights: vec![0.25, 0.35] }],
        };
        let mut output = Layer {
            bias_weight: 0.6,
            neurons: vec![Neuron { weights: vec![0.4, 0.45] },
                          Neuron { weights: vec![0.5, 0.55] }],
        };

        let ideal_output = vec![0.01, 0.99];

        let input = vec![0.05, 0.1];
        let lr = 0.5;

        let hidden_output = hidden.forward(&input);
        println!("hidden_output={:?}", hidden_output);
        assert_eq!(hidden_output, vec![0.5932699921071872, 0.5980868603322034]);

        let output_output = output.forward(&hidden_output);
        println!("output_output={:?}", output_output);
        assert_eq!(output_output, vec![0.7514661448872071, 0.773044520599256]);

        let output_e = squared_error_of_vec(&ideal_output, &output_output);
        println!("output_e={}", output_e);
        assert_eq!(output_e, 0.2984208620279517);

        let origional_output = output.clone();
        let err_derivatives = output.backward_output_layer(&hidden_output, &ideal_output, lr);

        assert_eq!(output.neurons[0].weights,
                   vec![0.35892205696077184, 0.4085885375830735]);
        assert_eq!(output.neurons[1].weights,
                   vec![0.511291150290211, 0.5613828252169398]);

        println!("err_derivatives={:?}", err_derivatives);

        hidden.backward_hidden_layer(&input, &origional_output, &err_derivatives, lr);

        for _ in 0..10000 {
            let origional_output = output.clone();
            let err_derivatives = output.backward_output_layer(&hidden_output, &ideal_output, lr);
            hidden.backward_hidden_layer(&input, &origional_output, &err_derivatives, lr);
        }

        let hidden_output = hidden.forward(&input);
        println!("new hidden_output={:?}", hidden_output);
        let output_output = output.forward(&hidden_output);
        println!("new output_output={:?}", output_output);
        let output_e = squared_error_of_vec(&ideal_output, &output_output);
        println!("new output_e={}", output_e);

        assert!(output_e < 0.0001);
    }
}
