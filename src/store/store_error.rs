
#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("The DATABASE_URL environment variable is not set, or the location is incorrect")]
    BadDBLocation(#[from] std::env::VarError),
    /// A wrapper around any SQLx Error
    #[error("SQLx error: {0}")]
    SQLxError(#[from] sqlx::Error),
    /// A general database decode error that can turn into a SQLx Error::Decode error
    #[error("Could not decode value from database: {0}")]
    DecodeError(String)
}

impl StoreError {
    pub fn into_sqlx_decode_error(self) -> sqlx::Error {
        sqlx::Error::Decode(Box::new(self))
    }
}
