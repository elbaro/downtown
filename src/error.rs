#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cannot read a file: {path}")]
    CannotReadFile {
        source: std::io::Error,
        path: String,
    },
    #[error("Input stream is clsoed")]
    InputStreamClosed,
    #[error("Input stream has an error")]
    InputStreamError { source: std::io::Error },
}
