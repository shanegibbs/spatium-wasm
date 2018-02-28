// Would like to see a graph
// Would like a custom name for tensors
// Groupings
// Awesome: Generate code to return results

#[cfg(test)]
mod tests {
    extern crate test;
    // use super::*;

    use ag;
    use ag::gradient_descent_ops::Optimizer;
    use ndarray;
    use ndarray::*;

    type NdArray = ndarray::Array<f32, ndarray::IxDyn>;

    // #[test]
    fn test_example_autograd() {}

    #[test]
    fn test_autograd() {
        let ref x = ag::placeholder(&[]);
        let ref y = ag::placeholder(&[]);
        let ref a = ag::placeholder(&[]);
        let ref z = 2 * x * x + 3 * y + 1 + 5 * a;

        // dz/dy
        let ref gy = ag::grad(&[z], &[y])[0];

        // dz/dx
        let ref gx = ag::grad(&[z], &[x])[0];

        // dz/da
        let ref ga = ag::grad(&[z], &[a])[0];

        let ref grads = ag::grad(&[z], &[y, x, a]);

        // ddz/dx (differentiates `z` again)
        let ref ggx = ag::grad(&[gx], &[x])[0];

        // evaluation of symbolic gradients
        println!("{}", gy.eval(&[])); // => 3.
        println!("{:?}", ga.eval(&[]));
        println!("{}", ggx.eval(&[])); // => 4.

        assert_eq!(3., gy.eval(&[])[IxDyn(&[])]);
        assert_eq!(8., gx.eval(&[(x, &arr0(2.).into_dyn())])[IxDyn(&[])]);
        // assert_eq!(5., ga.eval(&[])[IxDyn(&[])]);

        // let ref input = ndarray::arr0(2.);
        let ref input = NdArray::from_shape_vec(IxDyn(&[1]), vec![2.]).unwrap();

        let result = ag::eval(&[gy, gx, ga], &[(x, input)]);
        println!(
            "gy={:?},gx={:?},ga={:?}",
            result[0][IxDyn(&[])],
            result[1][IxDyn(&[0])],
            result[2][IxDyn(&[])],
        );

        let result = ag::eval(grads, &[(x, input)]);
        println!(
            "gy={:?},gx={:?},ga={:?}",
            result[0][IxDyn(&[])],
            result[1][IxDyn(&[0])],
            result[2][IxDyn(&[])],
        );

        // dz/dx requires to fill the placeholder `x`
        // println!("{}", gx.eval(&[(x, &ndarray::arr0(2.))])); // => 8.

        // return;

        let ref x = ag::placeholder(&[-1, 3]);
        let ref y = ag::placeholder(&[-1, 2]);
        // let ref w = ag::variable(ag::ndarray_ext::glorot_uniform(&[3, 2]));
        let ref w = ag::variable(arr2(&[[1.1, 1.2], [1.3, 1.4], [1.5, 1.6]]));
        let ref b = ag::variable(ag::ndarray_ext::zeros(&[1, 2]));
        let ref z = ag::matmul(x, w) + b;
        // let ref loss = ag::reduce_mean(&ag::square(&ag::sub(z, y)), &[0], false);
        let ref loss = ag::sparse_softmax_cross_entropy(z, y);

        // let ref params = [w, b];
        // let ref params = [w, b];
        let ref grads = ag::grad(&[loss], &[w, b]);
        // let ref _accuracy = ag::reduce_mean(&ag::equal(predictions, y), &[0], false);
        let mut adam = ag::gradient_descent_ops::Adam::default();
        let ref update_ops = adam.compute_updates(&[w, b], grads);
        // let ref update_ops =
        //     ag::ops::gradient_descent_ops::sgd::SGD { lr: 0.1 }.compute_updates(params, grads);

        let train_x = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let train_y = vec![0., 1., 1., 0.];

        let as_arr = NdArray::from_shape_vec;
        let x_train = as_arr(ndarray::IxDyn(&[2, 3]), train_x).unwrap();
        println!("x_train={:?}", x_train);
        let y_train = as_arr(ndarray::IxDyn(&[2, 2]), train_y).unwrap();
        println!("y_train={:?}", y_train);

        println!("\nTensor x = {:?} {}", x, x);
        println!("Tensor y = {:?} {}", y, y);
        println!("Tensor W = {:?} {}", w, w);
        println!("Tensor B = {:?} {}", b, b);
        println!("Tensor Z = {:?} {}", z, z);
        println!("Tensor loss = {:?} {}", loss, loss);
        println!("Tensor grads = {:?}", grads);

        macro_rules! print_model {
            () => {
                let result = ag::eval(&[x, y, w, b, z, loss, &grads[1]], &[(x, &x_train), (y, &y_train)]);
                println!("\nx\n{:?}", result[0]);
                println!("\ny\n{:?}", result[1]);
                println!("\nW\n{:?}", result[2]);
                println!("\nB\n{:?}", result[3]);
                println!("\nZ\n{:?}", result[4]);
                println!("\nloss\n{:?}", result[5]);
                println!("\ngrad0\n{:?}", result[6]);
            }
        }

        print_model!();
        // println!("Running update_ops");
        // ag::run(update_ops, &[(x, &x_train), (y, &y_train)]);
        // println!("\n\nSecond run");
        // print_model!();
    }
}
