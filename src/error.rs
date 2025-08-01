use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpeechApiError {
    #[error("WebSocket connection error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Connection timeout")]
    Timeout,

    #[error("Invalid audio format")]
    InvalidAudioFormat,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid session configuration: {0}")]
    InvalidConfig(String),

    #[error("Connection closed unexpectedly")]
    ConnectionClosed,
}

pub type Result<T> = std::result::Result<T, SpeechApiError>;
