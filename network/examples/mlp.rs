extern crate autograd as ag;
extern crate ndarray;

use self::ag::gradient_descent_ops::Optimizer;

// cargo run -p spatium-network --example mlp

fn main() {
    let inputs = 2usize;
    let outputs = 2usize;

    let inputs_i = inputs as isize;
    let outputs_i = outputs as isize;

    let (x_train, y_train) = dataset::load();

    assert_eq!(x_train.shape()[1], inputs);
    assert_eq!(y_train.shape()[1], 1);

    // -- graph def --
    let ref x = ag::placeholder(&[-1, inputs_i]);
    let ref y = ag::placeholder(&[-1, outputs_i]);
    let ref w = ag::variable(ag::ndarray_ext::glorot_uniform(&[inputs, outputs]));
    let ref b = ag::variable(ag::ndarray_ext::zeros(&[1, outputs]));
    let ref z = ag::matmul(x, w) + b;
    let ref loss = ag::sparse_softmax_cross_entropy(z, y);
    let ref loss_square = ag::square(loss);
    let ref mse = ag::reduce_mean(loss_square, &[0], false);
    let ref params = [w, b];
    let ref grads = ag::grad(&[loss], params);
    let ref predictions = ag::argmax(z, -1, true);
    let ref accuracy = ag::reduce_mean(&ag::equal(predictions, y), &[0], false);
    // let mut adam = ag::gradient_descent_ops::Adam::default();
    // let ref update_ops = adam.compute_updates(params, grads);
    let ref update_ops =
        ag::ops::gradient_descent_ops::sgd::SGD { lr: 0.01 }.compute_updates(params, grads);

    let max_epoch = 500;

    let epoc_ops: Vec<&ag::Tensor> = vec![&accuracy, &mse, &update_ops[0], &update_ops[1]];

    for epoch in 0..max_epoch {
        // ag::run(update_ops, &[(x, &x_train), (y, &y_train)]);
        // println!("loss: {}", loss_sum.eval(&[(x, &x_train), (y, &y_train)]));

        let result = ag::eval(&epoc_ops, &[(x, &x_train), (y, &y_train)]);
        let accuracy = &result[0][0];
        let mse = &result[1];
        println!("{} loss: {}, accuracy: {}", epoch, mse, accuracy);
        if *accuracy >= 1. {
            break;
        }

        // println!("finish epoch {}", epoch);
    }

    let result = ag::eval(
        &[x, y, w, b, z, loss, &grads[0], &grads[1], predictions],
        &[(x, &x_train), (y, &y_train)],
    );
    println!("\nx\n{:?}", result[0]);
    println!("\ny\n{:?}", result[1]);
    println!("\nW\n{:?}", result[2]);
    println!("\nB\n{:?}", result[3]);
    println!("\nZ\n{:?}", result[4]);
    println!("\nloss\n{:?}", result[5]);
    println!("\ngrad0\n{:?}", result[6]);
    println!("\ngrad1\n{:?}", result[7]);
    println!("\npredictions\n{:?}", result[8]);
}

pub mod dataset {
    extern crate ndarray;

    type NdArray = ndarray::Array<f32, ndarray::IxDyn>;

    pub fn load() -> (NdArray, NdArray) {
        // load dataset as `Vec`s
        let (train_x, num_image_train): (Vec<f32>, usize) =
            (vec![0., 0., 1., 0., 0., 1., 1., 1.], 4);
        let (train_y, num_label_train): (Vec<f32>, usize) = (vec![0., 0., 0., 1.], 4);

        // Vec to ndarray
        let as_arr = NdArray::from_shape_vec;
        let x_train = as_arr(ndarray::IxDyn(&[num_image_train, 2]), train_x).unwrap();
        let y_train = as_arr(ndarray::IxDyn(&[num_label_train, 1]), train_y).unwrap();
        (x_train, y_train)
    }
}
