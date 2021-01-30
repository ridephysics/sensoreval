#![allow(non_snake_case)]

pub fn unscented_transform<'a, Ss, Swm, Swc, Sq, A>(
    sigmas: &'a ndarray::ArrayBase<Ss, ndarray::Ix2>,
    Wm: &ndarray::ArrayBase<Swm, ndarray::Ix1>,
    Wc: &ndarray::ArrayBase<Swc, ndarray::Ix1>,
    Q: &ndarray::ArrayBase<Sq, ndarray::Ix2>,
    mean_fn: impl Fn(
        &ndarray::ArrayBase<Ss, ndarray::Ix2>,
        &ndarray::ArrayBase<Swm, ndarray::Ix1>,
    ) -> ndarray::Array1<A>,
    residual_fn: impl Fn(
        &ndarray::ArrayBase<ndarray::ViewRepr<&'a A>, ndarray::Ix1>,
        &ndarray::Array1<A>,
    ) -> ndarray::Array1<A>,
) -> (ndarray::Array1<A>, ndarray::Array2<A>)
where
    Ss: ndarray::Data<Elem = A>,
    Swm: ndarray::Data<Elem = A>,
    Swc: ndarray::Data<Elem = A>,
    Sq: ndarray::Data<Elem = A>,
    A: Copy
        + num_traits::identities::Zero
        + ndarray::ScalarOperand
        + std::ops::AddAssign
        + std::ops::Mul<A, Output = A>
        + std::fmt::Display,
{
    assert_eq!(Wm.dim(), Wc.dim());
    assert_eq!(sigmas.dim().0, Wm.dim());

    let x = mean_fn(sigmas, Wm);

    let shape = sigmas.shape();
    let mut P = ndarray::Array2::<A>::zeros((shape[1], shape[1]));
    assert_eq!(Q.dim(), P.dim());

    for k in 0..shape[0] {
        let yi = residual_fn(&sigmas.index_axis(ndarray::Axis(0), k), &x);
        let op = math::outer_product(&yi, &yi);
        let ops = &op * Wc[k];
        P += &ops;
    }

    P += Q;

    (x, P)
}

#[cfg(test)]
mod tests {
    use super::super::sigma_points::SigmaPoints;
    use super::*;
    use ndarray::array;

    #[test]
    fn ut() {
        let x = array![0.123, 0.789];
        let P = array![[1.0, 0.1], [0.1, 1.0]];
        let Q = array![[0.588, 1.175], [1.175, 2.35]];
        let fns = super::super::sigma_points::tests::LinFns::default();
        let points = super::super::sigma_points::MerweScaledSigmaPoints::new(2, 0.1, 2.0, 1.0, fns);
        let sigmas = points.sigma_points(&x, &P).unwrap();
        let Wc = points.weights_covariance();
        let Wm = points.weights_mean();

        let (xt, Pt) = unscented_transform(
            &sigmas,
            &Wm,
            &Wc,
            &ndarray::Array::zeros(P.dim()),
            |sigmas, mean| mean.dot(sigmas),
            |a, b| a - b,
        );
        testlib::assert_arr1_eq(&xt, &x);
        testlib::assert_arr2_eq(&Pt, &P);

        let (xt, Pt) = unscented_transform(
            &sigmas,
            &Wm,
            &Wc,
            &Q,
            |sigmas, mean| mean.dot(sigmas),
            |a, b| a - b,
        );
        testlib::assert_arr1_eq(&xt, &x);
        testlib::assert_arr2_eq(&Pt, &(P + Q));
    }
}
