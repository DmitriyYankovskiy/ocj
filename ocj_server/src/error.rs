use rand::{rngs::OsRng, TryRngCore};
use tokio_tungstenite;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("incorrect key")]
    IncorrectKey,

    #[error("ip not found")]
    IpNotFound,

    #[error("incorrect key")]
    IncorrectToken,
}

#[derive(Debug, thiserror::Error)]
pub enum ContestError {
    #[error("still in develop")]
    StillInDevelop,

    #[error("not started yet")]
    NotStartedYet,
    
    #[error("already ready")]
    AlreadyReady,

    #[error("already going")]
    AlreadyGoing,

    #[error("already finished")]
    AlreadyFinished,
}

#[derive(Debug, thiserror::Error)]
pub enum OcjError {
    #[error("not a single machine was found")]
    NoneMachineFound,

    #[error("local ip address error: {0:?}")]
    LocalIpAddress(#[from] local_ip_address::Error),

    #[error("tungstenite error {0:?}")]
    Tungstenite (#[from] tokio_tungstenite::tungstenite::Error),

    #[error("IO error {0:?}")]
    Io (#[from] tokio::io::Error),

    #[error("tokio join error {0:?}")]
    Join (#[from] tokio::task::JoinError),

    #[error("auth error {0:?}")]
    Auth(#[from] AuthError),

    #[error("env args <{0}> not found")]
    EnvArgsNotFound(&'static str),

    #[error("try rng core error {0:?}")]
    RngCore(<OsRng as TryRngCore>::Error),

    #[error("contest error {0:?}")]
    Contest(#[from] ContestError),

    #[error("system time error {0:?}")]
    SystemTime(#[from] std::time::SystemTimeError),
}

pub type Result<T> = std::result::Result<T, OcjError>;