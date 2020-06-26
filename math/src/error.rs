#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Linalg(#[from] ndarray_linalg::error::LinalgError),

    #[error("not positive semi-definite")]
    NotPositiveSemiDefinite,
    #[error("not square")]
    NotSquare,
    #[error("sigular matrix")]
    SingularMatrix,
    #[error("wrong vec len {0}")]
    WrongVecLen(usize),
}
