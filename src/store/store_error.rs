
#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("The DATABASE_URL environment variable is not set, or the location is incorrect")]
    BadDBLocation(#[from] std::env::VarError),
    #[error("SQLx error: {0}")]
    SQLxError(#[from] sqlx::Error)
}