//! A way to generate truly random numbers in the simulation.
//! This will simulate drawing entropy from physical devices, as the host will actually do this,
//! in order to supply the number.
//!
//! Provides:
//! - [get_random_u32]
//! - [get_random_u64]
//! - [get_random_bytes]
//!
//! # Examples
//!
//! ```rs
//! let seed = get_random_u32();
//! ```

use rand::{
    TryRng,
    rngs::{SysError, SysRng},
};
use thiserror::Error;

use crate::memory::Bytes;

#[derive(Error, Debug, Clone)]
pub enum RngError {
    #[error("SysError: {0}")]
    SysError(#[from] SysError),
}

pub type RngResult<T> = Result<T, RngError>;

#[must_use]
pub fn get_random_u32() -> RngResult<u32> {
    tracing::trace!("Getting random u32 from system");
    Ok(SysRng::default().try_next_u32()?)
}

#[must_use]
pub fn get_random_u64() -> RngResult<u64> {
    tracing::trace!("Getting random u64 from system");
    Ok(SysRng::default().try_next_u64()?)
}

pub fn get_random_bytes(bytes: &mut Bytes) -> RngResult<()> {
    tracing::trace!("Getting random bytes from system");
    SysRng::default().try_fill_bytes(bytes)?;
    Ok(())
}
