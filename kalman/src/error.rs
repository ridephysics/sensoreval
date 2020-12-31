#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Math(#[from] math::Error),
    #[error(transparent)]
    LinalgError(#[from] ndarray_linalg::error::LinalgError),

    #[error("can't convert float")]
    FloatConversion,
    #[error("invalid argument")]
    InvalidArgument,
    #[error("filters have different shapes")]
    DifferentFilterShapes,
    #[error("not enough filters")]
    NotEnoughFilters,
}
