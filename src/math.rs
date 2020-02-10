use std::ops::Mul;

pub const GRAVITY: f64 = 9.80665;

pub fn outer_product<Sa, Sb, Aa, Ab>(
    a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
    b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
) -> ndarray::Array<Ab, ndarray::Ix2>
where
    Sa: ndarray::Data<Elem = Aa>,
    Sb: ndarray::Data<Elem = Ab>,
    Ab: Clone + num_traits::identities::Zero + std::ops::Mul<Aa, Output = Ab>,
    Aa: Copy + ndarray::ScalarOperand,
{
    let mut res = ndarray::Array2::<Ab>::zeros((a.dim(), b.dim()));

    for i in 0..a.dim() {
        res.index_axis_mut(ndarray::Axis(0), i)
            .assign(&(b.mul(a[i])));
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn outer_product_test() {
        let res = outer_product(
            &ndarray::Array1::<f64>::ones(5),
            &ndarray::Array1::<f64>::linspace(-2.0, 2.0, 5),
        );
        crate::test::assert_arr2_eq(
            &res,
            &array![
                [-2., -1., 0., 1., 2.],
                [-2., -1., 0., 1., 2.],
                [-2., -1., 0., 1., 2.],
                [-2., -1., 0., 1., 2.],
                [-2., -1., 0., 1., 2.]
            ],
        );
    }
}
