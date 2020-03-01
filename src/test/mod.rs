use approx::assert_abs_diff_eq;
use ndarray::azip;

pub fn assert_arr1_eq<Sa, Sb>(
    a: &ndarray::ArrayBase<Sa, ndarray::Ix1>,
    b: &ndarray::ArrayBase<Sb, ndarray::Ix1>,
) where
    Sa: ndarray::Data<Elem = f64>,
    Sb: ndarray::Data<Elem = f64>,
{
    assert_eq!(a.dim(), b.dim());

    azip!((a in a, b in b) assert_abs_diff_eq!(a, b, epsilon=1.0e-6));
}

pub fn assert_arr2_eq<Sa, Sb>(
    a: &ndarray::ArrayBase<Sa, ndarray::Ix2>,
    b: &ndarray::ArrayBase<Sb, ndarray::Ix2>,
) where
    Sa: ndarray::Data<Elem = f64>,
    Sb: ndarray::Data<Elem = f64>,
{
    assert_eq!(a.dim(), b.dim());

    azip!((a in a.gencolumns(), b in b.gencolumns()) assert_arr1_eq(&a, &b));
}
