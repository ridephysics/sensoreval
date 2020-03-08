#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    CairoIo(cairo::IoError),
    SerdePickle(serde_pickle::error::Error),
    ExitStatus(std::process::ExitStatus),
    BinCode(bincode::Error),
    TomlDe(toml::de::Error),

    NoDataSet,
    SampleNotFound,
    EOF,
    UnsupportedConfigs,
    InvalidData,
    UnsupportedDatatype,
    NoHudRenderer,
    InvalidArgument,
    FloatConversion,
    BlenderRenderNotFound,
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<cairo::IoError> for Error {
    #[inline]
    fn from(e: cairo::IoError) -> Self {
        Error::CairoIo(e)
    }
}

impl From<serde_pickle::error::Error> for Error {
    #[inline]
    fn from(e: serde_pickle::error::Error) -> Self {
        Error::SerdePickle(e)
    }
}

impl From<std::process::ExitStatus> for Error {
    #[inline]
    fn from(e: std::process::ExitStatus) -> Self {
        Error::ExitStatus(e)
    }
}

impl From<bincode::Error> for Error {
    #[inline]
    fn from(e: bincode::Error) -> Self {
        Error::BinCode(e)
    }
}

impl From<toml::de::Error> for Error {
    #[inline]
    fn from(e: toml::de::Error) -> Self {
        Error::TomlDe(e)
    }
}
