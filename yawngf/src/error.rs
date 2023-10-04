use thiserror::Error;

use wgpu::{CreateSurfaceError, RequestDeviceError};
#[derive(Debug, Error)]
pub enum InitError {
    #[error("couldn't create a wgpu surface")]
    SurfaceCreation(#[from] CreateSurfaceError),
    #[error("no adapter found that matched preferred options")]
    NoAdapter,
    #[error("requesting a device failed")]
    NoDevice(#[from] RequestDeviceError),
}
