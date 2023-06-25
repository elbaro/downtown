#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cannot read a file: {path}")]
    CannotReadFile {
        source: std::io::Error,
        path: String,
    },
}
