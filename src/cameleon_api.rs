use cameleon::u3v;
use std::fmt::Error;
use cameleon::{Camera, CameleonError};
use thiserror::Error;

type U3vCamera = Camera<u3v::ControlHandle, u3v::StreamHandle>;

#[derive(Error, Debug)]
pub enum CameleonApiError {
    #[error("No Camera Found")]
    NoCameraFound,

    #[error("Feature Not Found: {0}")]
    FeatureNotFound(String),

    #[error("{0} Feature is not of type {1}")]
    IncorrectType(String, String),

    #[error("CameleonError: {0}")]
    Cameleon(#[from] CameleonError),
}

pub struct CameleonApi {
    pub camera: U3vCamera,
}

impl CameleonApi {
    pub fn initialize() -> Result<Self, CameleonError> {
        let mut cameras = u3v::enumerate_cameras()?;
        let mut camera = cameras.pop().ok_or(CameleonApiError::NoCameraFound)?;

        camera.open()?;
        camera.load_context()?;

        Ok(Self { camera })
    }

    // have to define featureinfo enum
    pub fn get_feature_info(&mut self, feature_name: &str) -> Result<FeatureInfo, CameleonError>;

    // @Olivia read here: https://docs.rs/cameleon/0.1.14/cameleon/genapi/index.html
    pub fn read_int_feature(&mut self, feature_name: &str, value: &str) -> Result<i64, CameleonApiError> {
        let mut params_ctxt = self.camera.params_ctxt()?;

        let node = params_ctxt
            .node(feature_name)
            .ok_or(CameleonApiError::FeatureNotFound(feature_name.to_string()))?;

        let int_node = node
            .as_integer(&params_ctxt)
            .ok_or(CameleonApiError::IncorrectType(feature_name.to_string(), "Integer".to_string()))?;

        if !int_node.is_readable(&mut params_ctxt)? {
            // todo
        }
        
        let value = int_node.value(&mut params_ctxt)?;
        Ok(value)

    }

    pub fn write_feature(&mut self, feature_name: &str) -> Result<String, CameleonError>;

    pub fn run_command(&mut self, feature_name: &str) -> Result<(), CameleonError>;

    pub fn recieve_frame(&mut self, timeout: Duration) -> Result<ToBeDetermined, CameleonError>;

    pub fn start_streaming(&mut self, capacity: usize) -> Result<(), CameleonError>;

    pub fn stop_streaming(&mut self) -> Result<(), CameleonError>;

    pub fn save_image(&self, frame, path, format: ) -> Result<(), CameleonError>;
}