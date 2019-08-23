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
}

impl From<serde_pickle::error::Error> for Error {
    #[inline]
    fn from(e: serde_pickle::error::Error) -> Error {
        Error {
            repr: ErrorRepr::SerdePickle(e),
        }
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Error {
        Error {
            repr: ErrorRepr::Io(e),
        }
    }
}

impl From<std::process::ExitStatus> for Error {
    #[inline]
    fn from(e: std::process::ExitStatus) -> Error {
        Error {
            repr: ErrorRepr::ExitStatus(e),
        }
    }
}

impl From<bincode::Error> for Error {
    #[inline]
    fn from(e: bincode::Error) -> Error {
        Error {
            repr: ErrorRepr::BinCode(e),
        }
    }
}

impl From<toml::de::Error> for Error {
    #[inline]
    fn from(e: toml::de::Error) -> Error {
        Error {
            repr: ErrorRepr::TomlDe(e),
        }
    }
}
