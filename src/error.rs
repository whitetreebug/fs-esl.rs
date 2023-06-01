use std::num;

use thiserror::Error;
use tokio::io;

#[derive(Debug, Error)]
pub enum EslError {
    #[error("InternalError")]
    InternalError(#[from] io::Error),

    #[error("ParseIntError")]
    ParseIntError(#[from] num::ParseIntError),
}
