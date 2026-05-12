use super::{error::*, ffi::*, utils::*};
use std::{
    ffi::{self, CStr, CString, c_char, c_void},
    mem::{self, MaybeUninit},
    ptr,
    os::raw,
};
use strum::FromRepr;

mod ffi {
    
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// ============================================================================
// Initialize API
// ============================================================================

pub struct SentechApi {
    stapi_table: *mut StApi_Functions_t, // Pointer to the API function table
    genapi_table: *mut GenApi_Functions_t, // Pointer to the GenApi function table
}

impl SentechApi {
    pub fn initialize() -> Result<Self, _EStApiCError_t> {
        let mut raw_api: PApiFunctions = ptr::null_mut();

        let err_code = unsafe { StApiCInitialize(STAPI_VERSION, &mut raw_api) };

        if err_code != _EStApiCError_t_StApiCError_NoError {
            return Err(err_code);
        }

        let stapi_table = unsafe { (*raw_api).StApi };
        let genapi_table = unsafe { (*raw_api).GenApi };

        Ok(Self {
            stapi_table,
            genapi_table,
        })
    }

    pub fn create_system(&self) -> Result<SystemHandle, _EStApiCError_t> {

        let mut handle: StApiHandle_t = unsafe { mem::zeroed() };

        let create_fn = unsafe { (*(*self.stapi_table).IStSystem).CreateIStSystem.unwrap() };

        let err = unsafe { create_fn(_EStSystemVendor_t_StSystemVendor_Default, _EStInterfaceType_t_StInterfaceType_All, &mut handle) };
        
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(SystemHandle { 
            ptr: handle, 
            api_table: self.stapi_table 
        })
    }
}

impl Drop for SentechApi {
    fn drop(&mut self) {
        unsafe {
            if let Some(terminate) = (*self.stapi_table).StApiCTerminate {
                terminate();
            }
        }
    }
}

// ============================================================================
// API Version
// ============================================================================
pub struct ApiVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl ApiVersion{
    pub fn get_version(&self) -> Result<(u32,u32,u32), _EStApiCError_t> {
        let mut api_version: u32 = 0;

        let get_version_fn = unsafe { (*(*self.api_table).StApi).GetStApiVersion.unwrap() };

        let err = unsafe { get_version_fn(&mut api_version) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        // 16842754 = 0x01010002
        let major   = (api_version >> 24) & 0xFF; 
        let minor   = (api_version >> 16) & 0xFF;
        let subminor   = api_version & 0xFFFF;

        assert_eq!(major, STAPI_VERSION_MAJOR);
        assert_eq!(minor, STAPI_VERSION_MINOR);
        assert_eq!(subminor, STAPI_VERSION_SUBMINOR);

        Ok((major, minor, subminor))
    }
}

impl Drop for ApiVersion {
    fn drop(&mut self) {
        // Nothing to clean up
    }
}

// ============================================================================
// System Handle
// ============================================================================

pub struct SystemHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

impl SystemHandle {
    pub fn update_interface_list(&self) -> Result<(), _EStApiCError_t> {
        let update_fn = unsafe { (*(*self.api_table).IStSystem).UpdateInterfaceList.unwrap() };

        let err = unsafe { update_fn(&self.ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(())
    }

    pub fn get_interface_count(&self) -> Result<u32, _EStApiCError_t> {
        let mut ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let mut count: u32 = 0;

        let get_count_fn = unsafe { (*(*self.api_table).IStSystem).GetInterfaceCount.unwrap() };

        let err = unsafe { get_count_fn(&self.ptr, &mut count) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(count)
    }

    pub fn get_interface(&self, index: usize) -> Result<InterfaceHandle, _EStApiCError_t> { //Index of the target interface from 0 to GetInterfaceCount()-1
        let mut ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_if = unsafe { (*(*self.api_table).IStSystem).GetIStInterface.unwrap() }; 

        let err = unsafe { get_if(&self.ptr, index, &mut ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(InterfaceHandle { ptr, api_table: self.api_table })
    }
}

impl Drop for SystemHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStSystem).Release {
                release(&mut self.ptr);
            }
        }
    }
}

// ============================================================================
// Interface Handle (IStInterface & IStInterfaceInfo)
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum InterfaceType {
    Unknown,
    USB3Vision,
    GigEVision,
    CoaXPress,
    All,
}

pub struct InterfaceHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}
    

impl InterfaceHandle {
    pass
}

impl Drop for InterfaceHandle {
    pass
}


// ============================================================================
// Device Handle (IStDevice & IStDeviceInfo)
// ============================================================================

pub struct DeviceHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

impl DeviceHandle {
    pass
}

impl Drop for DeviceHandle {
    pass
}


// ============================================================================
// Data Stream Handle (IStDataStream & IStDataStreamInfo)
// ============================================================================

pub struct DataStreamHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

impl DataStreamHandle {
    pass
}

impl Drop for DataStreamHandle {
    pass
}

// ============================================================================
// Start Sentech Camera
// ============================================================================

fn main() {
    let api = match SentechApi::initialize() {
        Ok(api) => api,
        Err(err) => {
            eprintln!("Failed to initialize API: {:?}", err);
            return;
        }
    }

    let system = match api.create_system() {
        Ok(system) => system,
        Err(err) => {
            eprintln!("Failed to create system: {:?}", err);
            return;
        }
    };

    if let Err(e) = system.update_interface_list() {
        eprintln!("Failed to update interface list: {:?}", e);
        return;
    }

    println!("Successfully initialized API, created system, and updated interfaces!");

}