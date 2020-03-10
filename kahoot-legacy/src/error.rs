pub type KahootResult<T> = Result<T, KahootError>;

#[derive(Debug)]
pub enum KahootError {
    Network,
    InvalidCode,
    InvalidStatus,
    Json,
    ChallengeDecode,
    MissingToken,

    NoConnection,

    Generic(&'static str),
}
