use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("event log error: {0}")]
    EventLog(#[from] crate::event_log::EventLogError),
    #[error("unknown action type: {0}")]
    UnknownActionType(String),
    #[error("unknown runtime mode: {0}")]
    UnknownRuntimeMode(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
