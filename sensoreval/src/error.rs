#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BinCode(#[from] bincode::Error),
    #[error("exit status: {0}")]
    ExitStatus(std::process::ExitStatus),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdePickle(#[from] serde_pickle::error::Error),
    #[error(transparent)]
    SensorevalUtils(#[from] sensoreval_utils::Error),
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    #[error("blender render not found")]
    BlenderRenderNotFound,
    #[error("EOF")]
    EOF,
    #[error("no dataset")]
    NoDataSet,
    #[error("no HUD renderer")]
    NoHudRenderer,
    #[error("sample not found")]
    SampleNotFound,
    #[error("sunsupported configs")]
    UnsupportedConfigs,
    #[error("unsupported datatype")]
    UnsupportedDatatype,
}
