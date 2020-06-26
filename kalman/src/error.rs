#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Math(#[from] math::Error),

    #[error("can't convert float")]
    FloatConversion,
    #[error("invalid argument")]
    InvalidArgument,
}
