#[derive(Debug, thiserror::Error)]
pub enum DocxMcpError {
    #[error("Document not found: {handle}")]
    DocumentNotFound { handle: String },

    #[error("Index out of bounds: {message} (index: {index}, valid range: 0..{max})")]
    IndexOutOfBounds {
        message: String,
        index: usize,
        max: usize,
    },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Engine error: {message}")]
    EngineError { message: String },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },
}
