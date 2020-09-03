use crate::Error;
use ndarray_linalg::eigh::Eigh;
use num_traits::float::Float;

fn check_parameters<Sm, Sc, Am, Ac>(
    dim: Option<usize>,
    mean: &ndarray::ArrayBase<Sm, ndarray::Ix1>,
    cov: &ndarray::ArrayBase<Sc, ndarray::Ix2>,
) -> Result<usize, Error>
where
    Sm: ndarray::RawData<Elem = Am>,
    Sc: ndarray::RawData<Elem = Ac>,
{
    let dim = dim.unwrap_or_else(|| mean.len());

    if mean.shape()[0] != dim {
        return Err(Error::WrongVecLen(dim));
    }

    if cov.shape() != [dim, dim] {
        return Err(Error::NotSquare);
    }

    Ok(dim)
}

fn eigvalsh_to_eps<A, S>(
    spectrum: &ndarray::ArrayBase<S, ndarray::Ix1>,
    cond: Option<A>,
    rcond: Option<A>,
) -> A
where
    S: ndarray::Data<Elem = A>,
    A: num_traits::Float + std::convert::From<f32>,
{
    let cond = if let Some(rcond) = rcond {
        rcond
    } else {
        cond.unwrap_or({
            let eps = A::epsilon();
            let factor = if eps >= f32::EPSILON.into() {
                1.0e3f32
            } else {
                1.0e6f32
            };

            let factor: A = std::convert::From::from(factor);

            factor * eps
        })
    };

    let mut max = A::zero();
    for v in spectrum.iter() {
        max = v.abs().max(max);
    }

    cond * max
}

fn pinv_1d<'a, I, A>(i: I, eps: &A) -> ndarray::Array1<A>
where
    I: Iterator<Item = &'a A>,
    A: 'a + num_traits::Float + std::convert::From<f32>,
{
    let zero: A = std::convert::From::from(0.0);
    let one: A = std::convert::From::from(1.0);
    i.map(|x| if x.abs() <= *eps { zero } else { one / *x })
        .collect()
}

#[allow(non_snake_case)]
#[derive(Debug)]
struct PSD<A> {
    rank: usize,
    U: ndarray::Array2<A>,
    log_pdet: A,
}

#[allow(non_snake_case)]
impl<A> PSD<A> {
    pub fn new<S>(
        M: &ndarray::ArrayBase<S, ndarray::Ix2>,
        cond: Option<<A as ndarray_linalg::Scalar>::Real>,
        rcond: Option<<A as ndarray_linalg::Scalar>::Real>,
        uplo: ndarray_linalg::UPLO,
        allow_singular: bool,
    ) -> Result<PSD<A>, Error>
    where
        A: ndarray_linalg::Scalar + ndarray_linalg::Lapack,
        S: ndarray::Data<Elem = A>,
        <A as ndarray_linalg::Scalar>::Real: From<f32>,
    {
        let (s, u) = M.eigh(uplo)?;
        let eps = eigvalsh_to_eps(&s, cond, rcond);

        let mut s_min = <A as ndarray_linalg::Scalar>::Real::max_value();
        for v in s.iter() {
            s_min = v.min(s_min);
        }
        if s_min < -eps {
            return Err(Error::NotPositiveSemiDefinite);
        }

        let d: ndarray::Array1<&<A as ndarray_linalg::Scalar>::Real> =
            s.iter().filter(|v| *v > &eps).collect();
        if d.len() < s.len() && !allow_singular {
            return Err(Error::SingularMatrix);
        }

        let s_pinv = pinv_1d(s.iter(), &eps);
        let U = u * s_pinv.map(|v| A::from_real(v.sqrt()));

        Ok(PSD {
            rank: d.len(),
            U,
            log_pdet: d.map(|v| A::from_real(v.ln())).sum(),
        })
    }
}

fn _logpdf<Sx, Sm, A>(
    x: &ndarray::ArrayBase<Sx, ndarray::Ix1>,
    mean: &ndarray::ArrayBase<Sm, ndarray::Ix1>,
    psd: &PSD<A>,
) -> A
where
    Sx: ndarray::Data<Elem = A>,
    Sm: ndarray::Data<Elem = A>,
    A: ndarray_linalg::Scalar + num_traits::FloatConst + std::convert::From<f32>,
{
    let dev = x - mean;
    let maha = dev.dot(&psd.U).map(|v| v.powi(2)).sum();

    let two: A = std::convert::From::from(2.0);
    let log_2pi = (two * A::PI()).ln();
    let n0p5: A = std::convert::From::from(-0.5);
    let rank: A = std::convert::From::from(psd.rank as f32);

    n0p5 * (rank * log_2pi + psd.log_pdet + maha)
}

pub fn logpdf<Sx, Sm, Sc, A>(
    x: &ndarray::ArrayBase<Sx, ndarray::Ix1>,
    mean: &ndarray::ArrayBase<Sm, ndarray::Ix1>,
    cov: &ndarray::ArrayBase<Sc, ndarray::Ix2>,
    allow_singular: bool,
) -> Result<A, Error>
where
    Sx: ndarray::Data<Elem = A>,
    Sm: ndarray::Data<Elem = A>,
    Sc: ndarray::Data<Elem = A>,
    A: ndarray_linalg::Scalar
        + ndarray_linalg::Lapack
        + num_traits::FloatConst
        + std::convert::From<f32>,
    <A as ndarray_linalg::Scalar>::Real: std::convert::From<f32>,
{
    let _dim = check_parameters(None, mean, cov)?;
    let psd = PSD::new(cov, None, None, ndarray_linalg::UPLO::Lower, allow_singular)?;
    let out = _logpdf(x, mean, &psd);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use ndarray::array;

    #[test]
    fn logpdf() {
        let x = array![1.0f64];
        let mean = array![1.0f64];
        let cov = array![[0.01f64]];
        let res = super::logpdf(&x, &mean, &cov, true).unwrap();
        assert_abs_diff_eq!(res, 1.3836465597893728);

        let res = res.exp();
        assert_abs_diff_eq!(res, 3.989422804014326);

        let x = array![1.0f64, 2.0];
        let mean = array![1.1f64, 2.0];
        let cov = array![[1.0f64, 2.0], [2.0, 5.0]];
        let res = super::logpdf(&x, &mean, &cov, false).unwrap();
        assert_abs_diff_eq!(res, -1.8628770664093455);

        let res = super::logpdf(&x, &mean, &cov, true).unwrap();
        assert_abs_diff_eq!(res, -1.8628770664093455);
    }
}
