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

pub trait Cross<Rhs> {
    type Output;
    fn cross(&self, rhs: &Rhs) -> Self::Output;
}

impl<A, S, S2> Cross<ndarray::ArrayBase<S2, ndarray::Ix1>> for ndarray::ArrayBase<S, ndarray::Ix1>
where
    S: ndarray::Data<Elem = A>,
    S2: ndarray::Data<Elem = A>,
    A: ndarray::LinalgScalar,
{
    type Output = ndarray::Array1<A>;

    fn cross(&self, rhs: &ndarray::ArrayBase<S2, ndarray::Ix1>) -> Self::Output {
        let a = self;
        let b = rhs;

        assert_eq!(a.dim(), (3));
        assert_eq!(b.dim(), (3));

        array![
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0]
        ]
    }
}

/// Source: https://stackoverflow.com/a/33920320/2035624
pub fn line_angle_2d(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let a = array![1.0, 0.0, 0.0];
    let b = array![x2 - x1, y2 - y1, 0.0];
    let n = array![0.0, 0.0, -1.0];

    b.cross(&a).dot(&n).atan2(a.dot(&b))
}

pub fn agm(mut a: f64, mut g: f64) -> f64 {
    if a <= 0.0 || g <= 0.0 {
        return 0.0;
    }

    loop {
        let a0 = a;
        let g0 = g;

        a = (a0 + g0) / 2.0;
        g = (a0 * g0).sqrt();

        if a == g || (a - g).abs() >= (a0 - g0).abs() {
            return a;
        }
    }
}

pub fn pendulum_period(r: f64, theta: f64, g: f64) -> f64 {
    2.0 * std::f64::consts::PI * (r / g).sqrt() / agm(1.0, (theta / 2.0).cos())
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

    #[test]
    fn line_angle_2d() {
        // 0deg
        assert_abs_diff_eq!(
            super::line_angle_2d(0.0, 0.0, 1.0, 0.0),
            0.0,
            epsilon = 1.0e-3
        );
        assert_abs_diff_eq!(
            super::line_angle_2d(0.0, 0.0, -1.0, 0.0),
            std::f64::consts::PI,
            epsilon = 1.0e-3
        );

        // +-45deg
        assert_abs_diff_eq!(
            super::line_angle_2d(0.0, 0.0, 1.0, 1.0),
            std::f64::consts::FRAC_PI_4,
            epsilon = 1.0e-3
        );
        assert_abs_diff_eq!(
            super::line_angle_2d(0.0, 0.0, 1.0, -1.0),
            -std::f64::consts::FRAC_PI_4,
            epsilon = 1.0e-3
        );

        // +-90deg
        assert_abs_diff_eq!(
            super::line_angle_2d(0.0, 0.0, 0.0, 1.0),
            std::f64::consts::FRAC_PI_2,
            epsilon = 1.0e-3
        );
        assert_abs_diff_eq!(
            super::line_angle_2d(0.0, 0.0, 0.0, -1.0),
            -std::f64::consts::FRAC_PI_2,
            epsilon = 1.0e-3
        );

        // +-135deg
        assert_abs_diff_eq!(
            super::line_angle_2d(0.0, 0.0, -1.0, 1.0),
            std::f64::consts::FRAC_PI_2 + std::f64::consts::FRAC_PI_4,
            epsilon = 1.0e-3
        );
        assert_abs_diff_eq!(
            super::line_angle_2d(0.0, 0.0, -1.0, -1.0),
            -std::f64::consts::FRAC_PI_2 - std::f64::consts::FRAC_PI_4,
            epsilon = 1.0e-3
        );
    }

    #[test]
    fn agm() {
        assert_abs_diff_eq!(super::agm(1.0, 2.0), 1.456791031046907, epsilon = 1.0e-15);
        assert_abs_diff_eq!(
            super::agm(12.345, 98.765),
            44.638129792342220,
            epsilon = 1.0e-15
        );
    }

    #[test]
    fn pendulum_period() {
        assert_abs_diff_eq!(
            super::pendulum_period(1.0, (1.0f64).to_radians(), 9.807),
            2.006411688292279,
            epsilon = 1.0e-15
        );
        assert_abs_diff_eq!(
            super::pendulum_period(1.0, (10.0f64).to_radians(), 9.807),
            2.010200020835200,
            epsilon = 1.0e-15
        );
        assert_abs_diff_eq!(
            super::pendulum_period(1.0, (30.0f64).to_radians(), 9.807),
            2.041302039079650,
            epsilon = 1.0e-15
        );
        assert_abs_diff_eq!(
            super::pendulum_period(1.0, (100.0f64).to_radians(), 9.807),
            2.472311992836434,
            epsilon = 1.0e-15
        );
    }
}
