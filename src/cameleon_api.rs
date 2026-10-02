use cameleon::u3v;
use std::fmt::Error;
use cameleon::{Camera, CameleonError};

type U3vCamera = Camera<u3v::ControlHandle, u3v::StreamHandle>;

#[derive(Debug, thiserror::Error)]
pub enum CameleonApiError {
    #[error("no Cameleon-compatible camera found")]
    NoCameraFound,
    #[error(transparent)]
    Cameleon(#[from] CameleonError),
}

pub struct CameleonApi {
    pub camera: U3vCamera,
}

impl CameleonApi {
    pub fn create_still_image_filer(&self) -> Result<(), Error> {
        /* TODO */
    }

    pub fn initialize() -> Result<Self, CameleonApiError> {
        let mut cameras = u3v::enumerate_cameras()?;
        let mut camera = cameras.pop().ok_or(CameleonApiError::NoCameraFound)?;
        camera.open()?;
        camera.load_context()?;

        Ok(Self { camera })
    }

    pub fn create_system() -> Result<(), Error> {
        /* TODO  */
    }
}