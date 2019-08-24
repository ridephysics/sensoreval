#[derive(Debug)]
pub struct Error {
    pub repr: ErrorRepr,
}

#[derive(Debug)]
pub enum ErrorRepr {
    Io(std::io::Error),
    SerdePickle(serde_pickle::error::Error),
    ExitStatus(std::process::ExitStatus),
    BinCode(bincode::Error),
    TomlDe(toml::de::Error),

    NoDataSet,
    SampleNotFound,
}

impl From<ErrorRepr> for Error {
    #[inline]
    fn from(e: ErrorRepr) -> Self {
        Self { repr: e }
    }
}

impl From<serde_pickle::error::Error> for Error {
    #[inline]
    fn from(e: serde_pickle::error::Error) -> Self {
        Self {
            repr: ErrorRepr::SerdePickle(e),
        }
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        Self {
            repr: ErrorRepr::Io(e),
        }
    }
}

impl From<std::process::ExitStatus> for Error {
    #[inline]
    fn from(e: std::process::ExitStatus) -> Self {
        Self {
            repr: ErrorRepr::ExitStatus(e),
        }
    }
}

impl From<bincode::Error> for Error {
    #[inline]
    fn from(e: bincode::Error) -> Self {
        Self {
            repr: ErrorRepr::BinCode(e),
        }
    }
}

impl From<toml::de::Error> for Error {
    #[inline]
    fn from(e: toml::de::Error) -> Self {
        Self {
            repr: ErrorRepr::TomlDe(e),
        }
    }
}
