use crate::Error;
use ndarray::array;

#[macro_export]
macro_rules! num2t {
    ($type:ty, $num:expr) => {
        <$type>::from($num).ok_or(Error::FloatConversion)?
    };
}

#[allow(non_snake_case)]
pub fn Q_discrete_white_noise<A>(dim: usize, dt: A, var: A) -> Result<ndarray::Array2<A>, Error>
where
    A: num_traits::float::Float + ndarray::ScalarOperand,
{
    Ok(match dim {
        2 => array![
            [num2t!(A, 0.25) * dt.powi(4), num2t!(A, 0.5) * dt.powi(3)],
            [num2t!(A, 0.5) * dt.powi(3), dt.powi(2)]
        ],
        3 => array![
            [
                num2t!(A, 0.25) * dt.powi(4),
                num2t!(A, 0.5) * dt.powi(3),
                num2t!(A, 0.5) * dt.powi(2)
            ],
            [num2t!(A, 0.5) * dt.powi(3), dt.powi(2), dt],
            [num2t!(A, 0.5) * dt.powi(2), dt, num2t!(A, 1.0)]
        ],
        4 => array![
            [
                (dt.powi(6)) / num2t!(A, 36),
                (dt.powi(5)) / num2t!(A, 12),
                (dt.powi(4)) / num2t!(A, 6),
                (dt.powi(3)) / num2t!(A, 6)
            ],
            [
                (dt.powi(5)) / num2t!(A, 12),
                (dt.powi(4)) / num2t!(A, 4),
                (dt.powi(3)) / num2t!(A, 2),
                (dt.powi(2)) / num2t!(A, 2)
            ],
            [
                (dt.powi(4)) / num2t!(A, 6),
                (dt.powi(3)) / num2t!(A, 2),
                dt.powi(2),
                dt
            ],
            [
                (dt.powi(3)) / num2t!(A, 6),
                (dt.powi(2)) / num2t!(A, 2),
                dt,
                num2t!(A, 1.0)
            ]
        ],
        _ => return Err(Error::InvalidArgument),
    } * var)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discrete_white_noise() {
        let q = Q_discrete_white_noise(2, 1.0, 1.0).unwrap();
        testlib::assert_arr2_eq(&q, &array![[0.25, 0.5], [0.5, 1.0]]);

        let q = Q_discrete_white_noise(3, 1.0, 1.0).unwrap();
        testlib::assert_arr2_eq(
            &q,
            &array![[0.25, 0.5, 0.5], [0.5, 1.0, 1.0], [0.5, 1.0, 1.0]],
        );
    }
}
