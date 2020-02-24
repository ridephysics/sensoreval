#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    SerdePickle(serde_pickle::error::Error),
    ExitStatus(std::process::ExitStatus),
    BinCode(bincode::Error),
    TomlDe(toml::de::Error),
    NativeRet(std::os::raw::c_int),
    NulError(std::ffi::NulError),

    NoDataSet,
    SampleNotFound,
    EOF,
    UnsupportedConfigs,
    InvalidData,
    UnsupportedDatatype,
    NoHudRenderer,
    InvalidArgument,
    FloatConversion,
    NoVideoFile,
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
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

impl From<std::os::raw::c_int> for Error {
    #[inline]
    fn from(e: std::os::raw::c_int) -> Self {
        Error::NativeRet(e)
    }
}

impl From<std::ffi::NulError> for Error {
    #[inline]
    fn from(e: std::ffi::NulError) -> Self {
        Error::NulError(e)
    }
}
