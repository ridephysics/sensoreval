#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("native return: {0}")]
    NativeReturn(std::os::raw::c_int),
}
