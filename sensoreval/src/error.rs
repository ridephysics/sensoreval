#[derive(thiserror::Error, Debug)]
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
    Linalg(#[from] ndarray_linalg::error::LinalgError),
    #[error(transparent)]
    SensorevalUtils(#[from] sensoreval_utils::Error),

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
}
