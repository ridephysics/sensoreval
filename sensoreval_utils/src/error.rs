use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("exit status: {0}")]
    ExitStatus(std::process::ExitStatus),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Plotly(#[from] plotly_types::Error),
    #[error(transparent)]
    SerdePickle(#[from] serde_pickle::error::Error),

    #[error("no row")]
    NoRow,
    #[error("row already exists")]
    RowAlreadyExists,
    #[error("row not found")]
    RowNotFound,
}
