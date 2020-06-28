mod error;
pub use error::Error;

pub mod multivariate;

pub const GRAVITY: f64 = 9.80665;

use ndarray::array;
use std::ops::Mul;

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

pub fn pymod<A>(n: A, m: A) -> A
where
    A: Copy + std::ops::Rem<Output = A> + std::ops::Add<Output = A>,
{
    ((n % m) + m) % m
}

pub fn normalize_angle<A>(mut x: A) -> A
where
    A: Copy
        + num_traits::cast::NumCast
        + num_traits::float::FloatConst
        + std::ops::Rem<Output = A>
        + std::ops::Add<Output = A>
        + std::cmp::PartialOrd
        + std::ops::Mul<A, Output = A>
        + std::ops::SubAssign,
{
    let pi = num_traits::float::FloatConst::PI();

    x = pymod(x, pi * A::from(2.0).unwrap());
    if x > pi {
        x -= pi * A::from(2.0).unwrap();
    }
    x
}

#[derive(Default)]
pub struct SinCosSum<A> {
    pub sin: A,
    pub cos: A,
}

impl<A> SinCosSum<A>
where
    A: std::ops::AddAssign + num_traits::float::Float,
{
    pub fn add(&mut self, x: A, w: A) {
        self.sin += x.sin() * w;
        self.cos += x.cos() * w;
    }

    pub fn avg(&self) -> A {
        self.sin.atan2(self.cos)
    }
}

#[allow(non_snake_case)]
pub fn tri_solve_sas(b: f64, c: f64, A: f64) -> (f64, f64) {
    let a = (b.powi(2) + c.powi(2) - 2.0 * b * c * A.cos()).sqrt();

    if b < c {
        let B = (A.sin() * b / a).asin();
        let C = std::f64::consts::PI - A - B;
        (B, C)
    } else {
        let C = (A.sin() * a / b).asin();
        let B = std::f64::consts::PI - A - C;
        (B, C)
    }
}

#[allow(non_snake_case)]
pub fn rot2d<S, A>(v: &ndarray::ArrayBase<S, ndarray::Ix1>, angle: A) -> ndarray::Array1<A>
where
    S: ndarray::Data<Elem = A>,
    A: num_traits::float::Float,
{
    assert_eq!(v.dim(), (2));

    array![
        v[0] * angle.cos() - v[1] * angle.sin(),
        v[0] * angle.sin() + v[1] * angle.cos(),
    ]
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use ndarray::array;

    extern crate blas_src;

    #[test]
    fn outer_product() {
        let res = super::outer_product(
            &ndarray::Array1::<f64>::ones(5),
            &ndarray::Array1::<f64>::linspace(-2.0, 2.0, 5),
        );
        testlib::assert_arr2_eq(
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

    #[test]
    fn pymod() {
        assert_eq!(super::pymod(-5, 4), 3);
        assert_eq!(super::pymod(5, 2), 1);
        assert_abs_diff_eq!(super::pymod(3.14f64, 0.7f64), 0.34f64, epsilon = 1.0e-6);
    }

    #[test]
    fn normalize_angle() {
        assert_abs_diff_eq!(
            super::normalize_angle((1.0f64 - 359.0f64).to_radians()),
            (2.0f64).to_radians(),
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn sincossum() {
        let mut sum = super::SinCosSum::default();
        sum.add(0.0f64, 1.0);
        sum.add(0.0f64, 1.0);
        sum.add((90.0f64).to_radians(), 1.0);
        assert_abs_diff_eq!(sum.avg(), (26.565f64).to_radians(), epsilon = 1.0e-6);

        let mut sum = super::SinCosSum::default();
        sum.add(0.0f64, 0.25);
        sum.add(0.0f64, 0.25);
        sum.add((90.0f64).to_radians(), 0.5);
        assert_abs_diff_eq!(sum.avg(), (45.0f64).to_radians(), epsilon = 1.0e-6);

        let mut sum = super::SinCosSum::default();
        sum.add((359.0f64).to_radians(), 1.0);
        sum.add((3.0f64).to_radians(), 1.0);
        assert_abs_diff_eq!(sum.avg(), (1.0f64).to_radians(), epsilon = 1.0e-6);
    }

    #[test]
    #[allow(non_snake_case)]
    fn tri_solve_sas() {
        let (B, C) = super::tri_solve_sas(5.0, 7.0, (49.0f64).to_radians());
        assert_abs_diff_eq!(B, (45.4f64).to_radians(), epsilon = 1.0e-3);
        assert_abs_diff_eq!(C, (85.6f64).to_radians(), epsilon = 1.0e-3);
    }

    #[test]
    fn rot2d() {
        let x = super::rot2d(&array![0.0, 1.0], std::f64::consts::FRAC_PI_2);
        testlib::assert_arr1_eq(&x, &array![-1.0, 0.0]);

        let x = super::rot2d(&array![0.0, 1.0], -std::f64::consts::FRAC_PI_2);
        testlib::assert_arr1_eq(&x, &array![1.0, 0.0]);
    }
}
