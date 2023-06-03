use std::num;

use thiserror::Error;
use tokio::{io, sync, time};

#[derive(Debug, Error)]
pub enum EslError {
    #[error("InternalError")]
    InternalError(#[from] io::Error),

    #[error("ParseIntError")]
    ParseIntError(#[from] num::ParseIntError),

    #[error("RecvError")]
    RecvError(#[from] sync::oneshot::error::RecvError),

    #[error("Elapsed")]
    ElapsedError(#[from] time::error::Elapsed),
}
