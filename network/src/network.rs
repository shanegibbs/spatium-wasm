use rand::Rng;

// TODO: remove all allocations in the network
// TODO: add back prop to layer bias?

#[derive(Debug, Clone)]
struct Network {
    layers: Vec<Layer>,
}

#[derive(Debug, Clone)]
struct Layer {
    bias_weight: f64,
    neurons: Vec<Neuron>,
}

#[derive(Debug, Clone)]
struct Neuron {
    weights: Vec<f64>,
}

#[derive(Debug, Clone)]
enum LayerBackpropOutput {
    Output(Vec<f64>), // ideal output
    Hidden(Layer, Vec<f64>), // layer forward and its deltas
}

pub trait ActivationFn {
    fn run(f64) -> f64;
    fn derivative(f64) -> f64;
}

// TODO: make activation fn generic
struct Sigmoid;
impl ActivationFn for Sigmoid {
    fn run(n: f64) -> f64 {
        1f64 / (1f64 + (-n).exp())
    }
    fn derivative(n: f64) -> f64 {
        n * (1f64 - n)
    }
}

impl Network {
    fn run(&self, input: &Vec<f64>) -> Vec<f64> {
        let mut output = input.clone();
        for layer in self.layers.iter() {
            output = layer.forward(&output);
        }
        output
    }
    fn run_with_err(&self, input: &Vec<f64>, ideal_output: &Vec<f64>) -> (Vec<f64>, f64) {
        let output = self.run(input);
        let e = squared_error_of_vec(&ideal_output, &output);
        (output, e)
    }
    fn new_train(&mut self, input: &Vec<f64>, ideal: &Vec<f64>, lr: f64) {
        // println!("ideal: {:?}", ideal);

        // forward pass, collecting layer outputs
        let mut inputs = vec![];
        let mut output = input.clone();
        inputs.push(input.clone());
        for layer in self.layers.iter() {
            output = layer.forward(&output);
            inputs.push(output.clone());
        }
        println!("Forward pass: {:?}", inputs);

        // calculate delta out each neuron
        let mut is_output_layer = true;
        let mut delta = vec![];
        let mut last_layer = self.layers.last().unwrap().to_owned();

        let mut new_layers = vec![];

        for (i, layer) in self.layers.iter().enumerate().rev() {
            if is_output_layer {
                println!("Calculating deltas for output (layer {})", i);
                is_output_layer = false;
                delta = layer.calculate_deltas_of_output_layer(&inputs[i + 1], &ideal);
                println!("delta {:?}", delta);
                let new_layer = layer.create_updated_layer_from_deltas(&inputs[i], &delta, lr);
                new_layers.push(new_layer);
                continue;
            }
            println!("Calculating deltas for layer {}", i);
            delta = layer.calculate_deltas_of_hidden_layer(&inputs[i - 1], &last_layer, &delta);
            let new_layer = layer.create_updated_layer_from_deltas(&inputs[i], &delta, lr);
            new_layers.push(new_layer);
            println!("delta {:?}", delta);
            last_layer = layer.clone();
        }

        self.layers = new_layers.into_iter().rev().collect();
    }
    fn train(&mut self, input: &Vec<f64>, ideal: &Vec<f64>, lr: f64) {
        let mut layers = self.layers.iter_mut().rev();

        // update output layer
        let mut last_layer = layers.next().expect("at least 1 layer");
        let mut last_layer_output = last_layer.forward(&input);
        let mut last_deltas = last_layer.backward_output_layer(input, ideal, lr);

        // update hidden layers
        while let Some(layer) = layers.next() {
            let layer_output = layer.forward(&last_layer_output);
            last_deltas =
                layer.backward_hidden_layer(&last_layer_output, &last_layer, &last_deltas, lr);

            last_layer = layer;
            last_layer_output = layer_output;
        }
    }
}

impl Layer {
    fn new<R: Rng>(gen: &mut Generator<f64, R>, inputs: usize, size: usize) -> Layer {
        let mut neurons = vec![];
        for _ in 0..size {
            neurons.push(Neuron::new(gen, inputs));
        }
        Layer {
            bias_weight: gen.next().unwrap(),
            neurons: neurons,
        }
    }
    fn forward(&self, input: &Vec<f64>) -> Vec<f64> {
        let mut output = vec![];
        for neuron in &self.neurons {
            let neuron_output = neuron.output(&input, self.bias_weight);
            let activation = Sigmoid::run(neuron_output);
            output.push(activation);
        }
        output
    }
    fn calculate_deltas_of_output_layer(&self, output: &Vec<f64>, ideal: &Vec<f64>) -> Vec<f64> {
        assert_eq!(ideal.len(), self.neurons.len());
        assert_eq!(output.len(), self.neurons.len());

        let mut deltas = vec![];
        for (i, _neuron) in self.neurons.iter().enumerate() {
            let neuron_error = output[i] - ideal[i];
            let derivative = Sigmoid::derivative(output[i]);
            let delta = neuron_error * derivative;
            deltas.push(delta);
        }
        deltas
    }
    fn calculate_deltas_of_hidden_layer(&self,
                                        output: &Vec<f64>,
                                        forward: &Layer,
                                        deltas: &Vec<f64>)
                                        -> Vec<f64> {
        let mut new_deltas = vec![];
        for (i, _neuron) in self.neurons.iter().enumerate() {
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

            let derivative = Sigmoid::derivative(output[i]);
            // println!("neuron_derivative={}", derivative);

            let delta = neuron_error * derivative;
            new_deltas.push(delta);
        }
        new_deltas
    }
    fn create_updated_layer_from_deltas(&self,
                                        input: &Vec<f64>,
                                        deltas: &Vec<f64>,
                                        lr: f64)
                                        -> Layer {
        let mut neurons = vec![];

        for (i, neuron) in self.neurons.iter().enumerate() {
            let delta = deltas[i];
            let mut new_weights = vec![];
            for (j, w) in neuron.weights.iter().enumerate() {
                let input = input[j];
                let x = delta * input;
                // println!("x={}", x);
                new_weights.push(w - (lr * x));
            }
            neurons.push(Neuron { weights: new_weights });
        }

        Layer {
            bias_weight: self.bias_weight,
            neurons: neurons,
        }
    }
    fn backprop(&mut self,
                input: &Vec<f64>,
                last: LayerBackpropOutput,
                lr: f64)
                -> LayerBackpropOutput {
        // assert_eq!(input.len(), ideal.len());
        let output = self.forward(input);
        let before_update = self.clone();

        // used to calculate errors on next layer
        let mut new_deltas = vec![];

        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            // println!("\nNeuron {}", i);
            let neuron_output = output[i];
            // println!("neuron_output={}", neuron_output);

            let neuron_error = match &last {
                &LayerBackpropOutput::Output(ref ideal) => neuron_output - ideal[i],
                &LayerBackpropOutput::Hidden(ref forward, ref deltas) => {
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
                    neuron_error
                }
            };
            // println!("neuron_error={}", neuron_error);

            // angle of the curve (what direction we should change in)
            let derivative = Sigmoid::derivative(neuron_output);
            // println!("neuron_derivative={}", derivative);

            let delta = neuron_error * derivative;
            new_deltas.push(delta);
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

        LayerBackpropOutput::Hidden(before_update, new_deltas)
    }
    // TODO: combine backward_output_layer and backward_hidden_layer
    fn backward_hidden_layer(&mut self,
                             input: &Vec<f64>,
                             forward: &Layer,
                             deltas: &Vec<f64>,
                             lr: f64)
                             -> Vec<f64> {
        // println!("Hidden layer");
        let output = self.forward(input);

        // used to calculate errors on next layer
        let mut new_deltas = vec![];

        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            let neuron_output = output[i];
            // println!("neuron_output={}", neuron_output);

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

            let derivative = Sigmoid::derivative(neuron_output);
            // println!("neuron_derivative={}", derivative);

            let delta = neuron_error * derivative;
            new_deltas.push(delta);
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

        new_deltas
    }
    fn backward_output_layer(&mut self, input: &Vec<f64>, ideal: &Vec<f64>, lr: f64) -> Vec<f64> {
        assert_eq!(input.len(), ideal.len());
        let output = self.forward(input);

        // used to calculate errors on next layer
        let mut new_deltas = vec![];

        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            // println!("\nNeuron {}", i);
            let neuron_output = output[i];
            // println!("neuron_output={}", neuron_output);

            // derivative wrt output
            let neuron_error = neuron_output - ideal[i];
            // println!("neuron_error={}", neuron_error);

            // derivative wrt output
            let derivative_wrt_output = Sigmoid::derivative(neuron_output);
            // println!("neuron_derivative={}", derivative);

            let delta = neuron_error * derivative_wrt_output;
            new_deltas.push(delta);
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

        new_deltas
    }
}

use rand::Generator;

impl Neuron {
    fn new<R: Rng>(gen: &mut Generator<f64, R>, inputs: usize) -> Neuron {
        Neuron { weights: gen.take(inputs).collect() }
    }
    fn output(&self, input: &Vec<f64>, bias: f64) -> f64 {
        assert_eq!(self.weights.len(), input.len());
        input
            .iter()
            .zip(&self.weights)
            .fold(bias, |sum, (i, w)| sum + i * w)
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
    extern crate test;

    use super::*;
    use self::test::Bencher;

    #[test]
    fn test_layer() {
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

    #[test]
    fn test_new_layer() {
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
        let backprop_output = output.backprop(&hidden_output,
                                              LayerBackpropOutput::Output(ideal_output.clone()),
                                              lr);

        assert_eq!(output.neurons[0].weights,
                   vec![0.35892205696077184, 0.4085885375830735]);
        assert_eq!(output.neurons[1].weights,
                   vec![0.511291150290211, 0.5613828252169398]);

        println!("backprop_output={:?}", backprop_output);

        hidden.backprop(&input, backprop_output, lr);

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

    #[test]
    fn test_network() {
        let mut net = Network {
            layers: vec![Layer {
                             bias_weight: 0.35,
                             neurons: vec![Neuron { weights: vec![0.15, 0.2] },
                                           Neuron { weights: vec![0.25, 0.35] }],
                         },
                         Layer {
                             bias_weight: 0.6,
                             neurons: vec![Neuron { weights: vec![0.4, 0.45] },
                                           Neuron { weights: vec![0.5, 0.55] }],
                         }],
        };

        let input = vec![0.05, 0.1];
        let ideal_output = vec![0.01, 0.99];
        let lr = 0.5;

        let (output, e_before_train) = net.run_with_err(&input, &ideal_output);
        println!("output={:?}", output);
        assert_eq!(output, vec![0.7514661448872071, 0.773044520599256]);
        assert_eq!(e_before_train, 0.2984208620279517);

        net.new_train(&input, &ideal_output, lr);

        let (output, e_after_train) = net.run_with_err(&input, &ideal_output);
        assert!(e_after_train < e_before_train);

        for _ in 0..10000 {
            net.new_train(&input, &ideal_output, lr);
        }

        let (output, e_after_lots_of_train) = net.run_with_err(&input, &ideal_output);
        println!("e_after_lots_of_train={:?}", e_after_lots_of_train);
        assert!(e_after_lots_of_train < 0.0001);
    }

    #[test]
    fn test_train_random_network() {
        use pcg_rand::Pcg32Basic;
        use rand::SeedableRng;

        let mut rnd = Pcg32Basic::from_seed([23, 54]);
        let mut gen = rnd.gen_iter();

        // Network Parameters
        let num_input = 5;
        let n_hidden_1 = 4;
        let n_hidden_2 = 3;
        let num_classes = 2;

        let mut net = Network {
            layers: vec![Layer::new(&mut gen, num_input, n_hidden_1),
                         Layer::new(&mut gen, n_hidden_1, n_hidden_2),
                         Layer::new(&mut gen, n_hidden_2, num_classes)],
        };
        let input = (&mut gen).take(num_input).collect();
        let ideal_output = (&mut gen).take(num_classes).collect();

        let lr = 0.5;

        let (output, e_before_train) = net.run_with_err(&input, &ideal_output);
        println!("e_before_train={:?}", e_before_train);

        net.new_train(&input, &ideal_output, lr);

        let (output, e_after_train) = net.run_with_err(&input, &ideal_output);
        assert!(e_after_train < e_before_train);

        for _ in 0..10000 {
            net.train(&input, &ideal_output, lr);
        }

        let (output, e_after_lots_of_train) = net.run_with_err(&input, &ideal_output);
        assert!(e_after_lots_of_train < 0.0001);
    }

    #[bench]
    fn bench_forward(b: &mut Bencher) {
        // getting         ~ 416,572 ns/iter (+/- 86,778)
        // tensorflow gets ~ 259,020 ns/iter

        use pcg_rand::Pcg32Basic;
        use rand::SeedableRng;

        let mut rnd = Pcg32Basic::from_seed([23, 54]);
        let mut gen = rnd.gen_iter();

        // Network Parameters
        let n_hidden_1 = 256; // 1st layer number of neurons
        let n_hidden_2 = 256; // 2nd layer number of neurons
        let num_input = 784; // MNIST data input (img shape: 28*28)
        let num_classes = 10; // MNIST total classes (0-9 digits)

        let net = Network {
            layers: vec![Layer::new(&mut gen, num_input, n_hidden_1),
                         Layer::new(&mut gen, n_hidden_1, n_hidden_2),
                         Layer::new(&mut gen, n_hidden_2, num_classes)],
        };
        let input = gen.take(num_input).collect();

        b.iter(|| net.run(&input));
    }
}
