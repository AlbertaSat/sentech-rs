use std::path::Path;
use std::time::{Duration, Instant};

use cameleon::genapi::GenApiError;

use cameleon::payload::ImageInfo; //METADATA ; checkout cameleon/src/payload.rs for more info on this struct
use cameleon::payload::Payload; //ACTUAL IMAGE DATA ; checkout cameleon/src/payload.rs for more info on this struct
use cameleon::payload::PayloadReceiver;
use cameleon::payload::PixelFormat;

use cameleon::u3v;
use cameleon::{Camera, CameleonError, StreamError};
use thiserror::Error;

use image::{GrayImage, ImageBuffer, RgbImage};

type U3vCamera = Camera<u3v::ControlHandle, u3v::StreamHandle>;


#[derive(Error, Debug)]
pub enum CameleonApiError {
    #[error("No Camera Found")]
    NoCameraFound,

    #[error("Feature Not Found: {0}")]
    FeatureNotFound(String),

    #[error("{0} Feature is not of type {1}")]
    IncorrectType(String, String),

    #[error("{0} Feature is not readable")]
    NotReadable(String),

    #[error("{0} Feature is not writable")]
    NotWritable(String),

    #[error("Not streaming")]
    NotStreaming,

    #[error("Failed to save image: {0}")]
    SaveError(String),

    #[error("Timed out waiting for frame")]
    Timeout,

    #[error("Payload contained no image")]
    NoImage,

    #[error("Generic Error")]
    GenericError,

    #[error("CameleonError: {0}")]
    Cameleon(#[from] CameleonError),   // fixes every error mentioning From<CameleonError>

    #[error("GenApiError: {0}")]
    GenApi(#[from] GenApiError),       // fixes every error mentioning From<GenApiError>
}

pub struct Frame { // need frame struct for image metadata/data instead of it die in buffer
    pub payload: Payload, //this is the actual image data, which is a Vec<u8> in cameleon/src/payload.rs
        // notes on payload; can use payload.image() to get the image data as a slice of u8, and payload.image_info() to get the image metadata as an ImageInfo struct
        // can get payload.timestamp() to get the timestamp of the image, and payload.id() to get the id of the image
        // READ THIS FILE cameleon/src/payload.rs at  https://github.com/cameleon-rs/cameleon/blob/main/cameleon/src/payload.rs 
}

impl Frame {
    pub fn info(&self) -> &ImageInfo {
        self.payload.image_info().expect("checked in receive_frame")
    }
    pub fn data(&self) -> &[u8] {
        self.payload.image().expect("checked in receive_frame")
    }
    pub fn id(&self) -> u64 { self.payload.id() }
    pub fn timestamp(&self) -> Duration { self.payload.timestamp() }
}

pub struct CameleonApi {
    pub camera: U3vCamera,
    payload_rx: Option<PayloadReceiver>, //Payloadreceiver is a channel receiver for Payloads, which are the actual image data and metadata. This is used to receive frames from the camera when streaming
    // note that payload_rx should only be private, and it is Option bc Some whilst streaming, None else.
}

impl CameleonApi {
    pub fn initialize() -> Result<Self, CameleonApiError> {
        println!("Initializing Cameleon API...");
        println!("Enumerating cameras...");
        let mut cameras = u3v::enumerate_cameras()?;
        println!("Found {} cameras", cameras.len());
        let mut camera = cameras.pop().ok_or(CameleonApiError::NoCameraFound)?;
        println!("Opening camera...");
        camera.open()?;
        println!("Loading context...");
        camera.load_context()?;
        println!("Camera initialized successfully.");

        Ok(Self { camera , payload_rx: None })
    }

    pub fn get_feature_type(&mut self, feature_name: &str) -> Result<String, CameleonApiError> {
        let params_ctxt = self.camera.params_ctxt()?;

        let node = params_ctxt
            .node(feature_name)
            .ok_or(CameleonApiError::FeatureNotFound(feature_name.to_string()))?;

        if node.as_integer(&params_ctxt).is_some() {
            return Ok("Integer".to_string());
        }

        if node.as_float(&params_ctxt).is_some() {
            return Ok("Float".to_string());
        }

        if node.as_boolean(&params_ctxt).is_some() {
            return Ok("Boolean".to_string());
        }

        if node.as_enumeration(&params_ctxt).is_some() {
            return Ok("Enumeration".to_string());
        }

        if node.as_string(&params_ctxt).is_some() {
            return Ok("String".to_string());
        }

        if node.as_command(&params_ctxt).is_some() {
            return Ok("Command".to_string());
        }

        Err(CameleonApiError::GenericError)
    }

    // // have to define featureinfo enum
    // pub fn get_feature_info(&mut self, feature_name: &str) -> Result<FeatureInfo, CameleonError>;

    // @Olivia read here: https://docs.rs/cameleon/0.1.14/cameleon/genapi/index.html
    pub fn read_int_feature(&mut self, feature_name: &str) -> Result<i64, CameleonApiError> {
        println!("Reading integer feature: {}", feature_name);
        let mut params_ctxt = self.camera.params_ctxt()?;

        let node = params_ctxt
            .node(feature_name)
            .ok_or(CameleonApiError::FeatureNotFound(feature_name.to_string()))?;

        let int_node = node
            .as_integer(&params_ctxt)
            .ok_or(CameleonApiError::IncorrectType(feature_name.to_string(), "Integer".to_string()))?;

        if !int_node.is_readable(&mut params_ctxt)? {
            return Err(CameleonApiError::NotReadable(feature_name.to_string()));
      
        }
        
        let value = int_node.value(&mut params_ctxt)?;
        println!("Read integer feature: {} = {}", feature_name, value);
        Ok(value)

    }

    pub fn read_feature(&mut self, feature_name: &str) -> Result<String, CameleonApiError> {
        println!("Reading feature: {}", feature_name);
        let mut params_ctxt = self.camera.params_ctxt()?;

        let node = params_ctxt
            .node(feature_name)
            .ok_or(CameleonApiError::FeatureNotFound(feature_name.to_string()))?;

        let string_node = node
            .as_string(&params_ctxt)
            .ok_or(CameleonApiError::IncorrectType(feature_name.to_string(), "String".to_string()))?;

        if !string_node.is_readable(&mut params_ctxt)? {
            return Err(CameleonApiError::NotReadable(feature_name.to_string()));
        }

        let value = string_node.value(&mut params_ctxt)?;
        println!("Read feature: {} = {:?}", feature_name, value);
        Ok(value)
    }

    pub fn write_feature(&mut self, feature_name: &str, value: &str) -> Result<(), CameleonApiError> {
        println!("Writing feature: {} = {}", feature_name, value);
        let mut params_ctxt = self.camera.params_ctxt()?;

        let node = params_ctxt
            .node(feature_name)
            .ok_or(CameleonApiError::FeatureNotFound(feature_name.to_string()))?;

        let string_node = node
            .as_string(&params_ctxt)
            .ok_or(CameleonApiError::IncorrectType(feature_name.to_string(), "String".to_string()))?;

        if !string_node.is_writable(&mut params_ctxt)? {
            return Err(CameleonApiError::NotWritable(feature_name.to_string()));
        }

        string_node.set_value(&mut params_ctxt, value.to_string())?;
        println!("Successfully wrote feature: {} = {}", feature_name, value);
        Ok(())
    }

    pub fn write_int_feature(&mut self, feature_name: &str, value: i64) -> Result<(), CameleonApiError> {
        println!("Writing integer feature: {} = {}", feature_name, value);
        let mut ctxt = self.camera.params_ctxt()?;

        let node = ctxt
            .node(feature_name)
            .ok_or_else(|| CameleonApiError::FeatureNotFound(feature_name.to_string()))?;

        let int_node = node.as_integer(&ctxt).ok_or_else(|| {
            CameleonApiError::IncorrectType(feature_name.to_string(), "Integer".to_string())
        })?;

        if !int_node.is_writable(&mut ctxt)? {
            return Err(CameleonApiError::NotWritable(feature_name.to_string()));
        }

        int_node.set_value(&mut ctxt, value)?;
        println!("Successfully wrote integer feature: {} = {}", feature_name, value);
        Ok(())
    }

    pub fn run_command(&mut self, feature_name: &str) -> Result<(), CameleonApiError> { 
        println!("Running command feature: {}", feature_name);
        let mut ctxt = self.camera.params_ctxt()?;

        let node = ctxt
            .node(feature_name)
            .ok_or_else(|| CameleonApiError::FeatureNotFound(feature_name.to_string()))?;

        let cmd = node.as_command(&ctxt).ok_or_else(|| {
            CameleonApiError::IncorrectType(feature_name.to_string(), "Command".to_string())
        })?;

        cmd.execute(&mut ctxt)?;
        println!("Successfully executed command feature: {}", feature_name);    
        Ok(())
    }


    pub fn recieve_frame(&mut self, timeout: Duration) -> Result<Frame, CameleonApiError>{
        // use the self.payload_rx, which should be populated to Some(rx). otherwise it is None, but in that case you shouldnt call this function
        // can reference https://github.com/cameleon-rs/cameleon/blob/main/cameleon/src/camera.rs
        let payload_rx = self.payload_rx.as_ref().ok_or(CameleonApiError::NotStreaming)?;

        // cameleon has no recv-with-timeout, so poll try_recv until the deadline.
        // try_recv returns StreamError::ReceiveError when the channel is just empty.
        let start = Instant::now();
        let payload = loop {
            match payload_rx.try_recv() {
                Ok(payload) => break payload,
                Err(StreamError::ReceiveError(_)) if start.elapsed() < timeout => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(StreamError::ReceiveError(_)) => return Err(CameleonApiError::Timeout),
                Err(e) => return Err(CameleonError::from(e).into()),
            }
        };

        // Frame::info()/data() unwrap these, so reject payloads without an image here.
        if payload.image().is_none() || payload.image_info().is_none() {
            payload_rx.send_back(payload);
            return Err(CameleonApiError::NoImage);
        }

        println!(
            "payload received! block_id: {:?}, timestamp: {:?}",
            payload.id(),
            payload.timestamp()
        );
        Ok(Frame { payload })
    }

    pub fn start_streaming(&mut self, capacity: usize) -> Result<(), CameleonApiError> {
        let rx = self.camera.start_streaming(capacity)?; //capacity== num frames can be buffered in channel (if cap == 0, unbounded and will never drop but then the cpu goes bonkers trying to hold onto bazillion frames, so always set it to be finite and keep it small like 3 )
        self.payload_rx = Some(rx);
        Ok(())
    }

    pub fn stop_streaming(&mut self) -> Result<(), CameleonApiError> {
        self.camera.stop_streaming()?;
        self.payload_rx = None;
        Ok(())
    }

    pub fn save_image(&self, frame: &Frame, path: &Path, format: &str) -> Result<(), CameleonApiError>{
        // image() excludes chunk data that payload() would include
        let image_data = frame.payload.image().ok_or(CameleonApiError::NoImage)?;
        let info = frame.payload.image_info().ok_or(CameleonApiError::NoImage)?;
        let width = u32::try_from(info.width).map_err(|e| CameleonApiError::SaveError(e.to_string()))?;
        let height = u32::try_from(info.height).map_err(|e| CameleonApiError::SaveError(e.to_string()))?;

        let result = match info.pixel_format {
            PixelFormat::Mono8 => {
                let img: GrayImage = ImageBuffer::from_raw(width, height, image_data.to_vec())
                    .ok_or(CameleonApiError::NoImage)?;
                img.save(path)
            }
            PixelFormat::RGB8 => {
                let img: RgbImage = ImageBuffer::from_raw(width, height, image_data.to_vec())
                    .ok_or(CameleonApiError::NoImage)?;
                img.save(path)
            }
            other => {
                return Err(CameleonApiError::SaveError(format!(
                    "unsupported pixel format: {other:?}"
                )))
            }
        };
        result.map_err(|e| CameleonApiError::SaveError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_feature_type() {
        let mut api = CameleonApi::initialize()
            .expect("Failed to initialize Sentech Cameleon API");

        let feature_type = api
            .get_feature_type("Sharpness")
            .expect("Failed to get feature type");

        println!("Sharpness type: {}", feature_type);

        assert!(
            feature_type == "Integer"
        );
    }

    #[test]
    fn test_read_feature() {
        let mut api = CameleonApi::initialize()
            .expect("Failed to initialize Sentech CameleonAPI");

        let value = api
            .read_int_feature("Sharpness")
            .expect("Failed to read feature: Sharpness");

        println!("Sharpness: {}", value);
    }

    #[test]
    fn test_write_int_feature() {
        let mut api = CameleonApi::initialize()
            .expect("Failed to initialize Sentech API");

        let feature_name = "Sharpness";
        let original_value = api
            .read_int_feature(feature_name)
            .expect("Failed to read original Width");

        let test_value = original_value - 1;

        api.write_int_feature(feature_name, test_value)
            .expect("Failed to write Width");

        let new_value = api
            .read_int_feature(feature_name)
            .expect("Failed to read Width after writing");

        assert_eq!(
            new_value, test_value,
            "Width was not changed to the value we wrote"
        );

        // Restore the original value.
        api.write_int_feature(feature_name, original_value)
            .expect("Failed to restore original Width");
    }

     #[test]
    fn test_start_and_stop_streaming() {
        // Initialize the camera
        let mut api = CameleonApi::initialize()
            .expect("Failed to initialize Sentech API");

        // Start streaming with a small buffer
        api.start_streaming(3)
            .expect("Failed to start streaming");

        // Make sure the receiver was created
        assert!(
            api.payload_rx.is_some(),
            "Payload receiver should exist after starting streaming"
        );

        // Stop streaming
        api.stop_streaming()
            .expect("Failed to stop streaming");

        // Make sure the receiver was cleared
        assert!(
            api.payload_rx.is_none(),
            "Payload receiver should be None after stopping streaming"
        );
    }
}