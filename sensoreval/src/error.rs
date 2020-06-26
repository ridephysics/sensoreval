use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    CairoIo(#[from] cairo::IoError),
    #[error(transparent)]
    SerdePickle(#[from] serde_pickle::error::Error),
    #[error("exit status: {0}")]
    ExitStatus(std::process::ExitStatus),
    #[error(transparent)]
    BinCode(#[from] bincode::Error),
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    #[error(transparent)]
    Plotly(#[from] plotly_types::Error),
    #[error(transparent)]
    Linalg(#[from] ndarray_linalg::error::LinalgError),

    #[error("no dataset")]
    NoDataSet,
    #[error("sample not found")]
    SampleNotFound,
    #[error("EOF")]
    EOF,
    #[error("sunsupported configs")]
    UnsupportedConfigs,
    #[error("invalid data")]
    InvalidData,
    #[error("unsupported datatype")]
    UnsupportedDatatype,
    #[error("no HUD renderer")]
    NoHudRenderer,
    #[error("invalid argument")]
    InvalidArgument,
    #[error("float conversion")]
    FloatConversion,
    #[error("blender render not found")]
    BlenderRenderNotFound,
    #[error("row already exists")]
    RowAlreadyExists,
    #[error("no row")]
    NoRow,
    #[error("row not found")]
    RowNotFound,

    #[error("not positive semi-definite")]
    NotPositiveSemiDefinite,
    #[error("sigular matrix")]
    SingularMatrix,
    #[error("wrong vec len {0}")]
    WrongVecLen(usize),
    #[error("not square")]
    NotSquare,
}
