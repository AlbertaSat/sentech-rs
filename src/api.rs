use super::{error::*, ffi::*, utils::*};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    mem,
    ptr,
    os::raw,
};
use strum::FromRepr;

// ============================================================================
// Initialize API
// ============================================================================

pub struct SentechApi {
    stapi_table: *mut StApi_Functions_t, // Pointer to the API function table
    genapi_table: *mut GenApi_Functions_t, // Pointer to the GenApi function table
}

pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub subminor: u32,
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

    // need to create system handle first to access the system-level functions like updating interface list, getting interface count, etc.
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

    pub fn get_version(&self) -> Result<ApiVersion, _EStApiCError_t> {
        let mut api_version: u32 = 0;

        let get_version_fn = unsafe { (*self.stapi_table).GetStApiVersion.unwrap() };

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

        Ok(ApiVersion { major, minor, subminor })
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
// System Handle
// ============================================================================

pub struct SystemHandle { 
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct SystemInfoHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

impl SystemHandle {

        pub fn get_ist_port(&self) -> Result<PortHandle, _EStApiCError_t> {
        let mut port_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_port = unsafe { (*(*self.api_table).IStSystem).GetIStPort.unwrap() };

        let err = unsafe { get_port(ptr::addr_of!(self.ptr) as *mut _, &mut port_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(PortHandle {
            ptr: port_ptr,
            api_table: self.api_table,
        })
    }

    pub fn get_ist_system_info(&self) -> Result<SystemInfoHandle, _EStApiCError_t> {
        let mut sys_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_sys_info = unsafe { (*(*self.api_table).IStSystem).GetIStSystemInfo.unwrap() };

        let err = unsafe { get_sys_info(ptr::addr_of!(self.ptr) as *mut _, &mut sys_info_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(SystemInfoHandle {
            ptr: sys_info_ptr,
            api_table: self.api_table,
        })
    }

    pub fn update_interface_list(&self) -> Result<bool, _EStApiCError_t> {
        let update_iface_list = unsafe { (*(*self.api_table).IStSystem).UpdateInterfaceList.unwrap() };

        let mut reval: bool8_t = 0;
        let err = unsafe { update_iface_list(ptr::addr_of!(self.ptr) as *mut _, &mut reval) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(reval != 0)
    }

    pub fn get_interface_count(&self) -> Result<u32, _EStApiCError_t> {
        let mut count: u32 = 0;

        let get_iface_count = unsafe { (*(*self.api_table).IStSystem).GetInterfaceCount.unwrap() };
        
        let err = unsafe { get_iface_count(ptr::addr_of!(self.ptr) as *mut _, &mut count) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(count)
    }

    pub fn get_interface(&self, index: usize) -> Result<InterfaceHandle, _EStApiCError_t> { //Index of the target interface from 0 to GetInterfaceCount()-1
        let mut iface_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_iface = unsafe { (*(*self.api_table).IStSystem).GetIStInterface.unwrap() }; 

        let err = unsafe { get_iface(ptr::addr_of!(self.ptr) as *mut _, index, &mut iface_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(InterfaceHandle { ptr: iface_ptr, api_table: self.api_table })
    }

    pub fn create_first_ist_device(&self, access_flag: DeviceAccess) -> Result<DeviceHandle, _EStApiCError_t> {
        let mut device_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let create_device = unsafe { (*(*self.api_table).IStSystem).CreateFirstIStDevice.unwrap() };

        let err = unsafe {
            create_device(
                ptr::addr_of!(self.ptr) as *mut _,
                access_flag as u32,
                &mut device_ptr,
            )
        };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(DeviceHandle {
            ptr: device_ptr,
            api_table: self.api_table,
        })
    }
}

impl SystemInfoHandle {
    pub fn get_system_id(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_sys_id = unsafe { (*(*self.api_table).IStSystemInfo).GetIDA.unwrap() };
        let err = unsafe { get_sys_id(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_mode(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_mode = unsafe { (*(*self.api_table).IStSystemInfo).GetModelA.unwrap() };
        let err = unsafe { get_mode(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_version(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_sys_name = unsafe { (*(*self.api_table).IStSystemInfo).GetVersionA.unwrap() };
        let err = unsafe { get_sys_name(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_sys_name = unsafe { (*(*self.api_table).IStSystemInfo).GetNameA.unwrap() };
        let err = unsafe { get_sys_name(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_path_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_sys_name = unsafe { (*(*self.api_table).IStSystemInfo).GetPathNameA.unwrap() };
        let err = unsafe { get_sys_name(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_display_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_sys_name = unsafe { (*(*self.api_table).IStSystemInfo).GetDisplayNameA.unwrap() };
        let err = unsafe { get_sys_name(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
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

impl Drop for SystemInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for system info in the API
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

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum DeviceAccess {
    Unknown = DEVICE_ACCESS_FLAGS_DEVICE_ACCESS_UNKNOWN,
    None = DEVICE_ACCESS_FLAGS_DEVICE_ACCESS_NONE,
    ReadOnly = DEVICE_ACCESS_FLAGS_DEVICE_ACCESS_READONLY,
    Control = DEVICE_ACCESS_FLAGS_DEVICE_ACCESS_CONTROL,
    Exclusive = DEVICE_ACCESS_FLAGS_DEVICE_ACCESS_EXCLUSIVE,
    CustomId = DEVICE_ACCESS_FLAGS_DEVICE_ACCESS_CUSTOM_ID,
}

pub struct InterfaceHandle { 
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct InterfaceInfoHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct InterfaceInfo {
    pub id: String,
    pub name: String,
    pub interface_handle: InterfaceHandle,
    pub interface_type: InterfaceType,
}

    
impl InterfaceHandle {

    pub fn get_ist_system(&self) -> Result<SystemHandle, _EStApiCError_t> {
        let mut system_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_system = unsafe { (*(*self.api_table).IStInterface).GetIStSystem.unwrap() };

        let err = unsafe { get_system(ptr::addr_of!(self.ptr) as *mut _, &mut system_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(SystemHandle { ptr: system_ptr, api_table: self.api_table })
    }

    pub fn get_interface_info(&self) -> Result<InterfaceInfoHandle, _EStApiCError_t> {
        let mut iface_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_iface_info = unsafe { (*(*self.api_table).IStInterface).GetIStInterfaceInfo.unwrap() };

        let err = unsafe { get_iface_info(ptr::addr_of!(self.ptr) as *mut _, &mut iface_info_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(InterfaceInfoHandle {
            ptr: iface_info_ptr,
            api_table: self.api_table,
        })
    }
 
    pub fn update_device_list(&self) -> Result<bool, _EStApiCError_t> {
        let update_dev_list = unsafe { (*(*self.api_table).IStInterface).UpdateDeviceList.unwrap() };

        let mut updated: bool8_t = 0;
        let err = unsafe { update_dev_list(ptr::addr_of!(self.ptr) as *mut _, &mut updated) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(updated != 0)
    }
 
    pub fn get_device_count(&self) -> Result<u32, _EStApiCError_t> {
        let mut count: u32 = 0;

        let get_dev_count = unsafe { (*(*self.api_table).IStInterface).GetDeviceCount.unwrap() };
        
        let err = unsafe { get_dev_count(ptr::addr_of!(self.ptr) as *mut _, &mut count) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(count)
    }
 
    pub fn get_ist_device_info(&self, index: usize) -> Result<DeviceInfoHandle, _EStApiCError_t> {
        let mut dev_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_dev_info = unsafe { (*(*self.api_table).IStInterface).GetIStDeviceInfo.unwrap() };

        let err = unsafe { get_dev_info(ptr::addr_of!(self.ptr) as *mut _, index, &mut dev_info_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(DeviceInfoHandle {
            ptr: dev_info_ptr,
            api_table: self.api_table,
        })
    }

    pub fn device_available(&self, index: usize, access_flag: DeviceAccess) -> Result<bool, _EStApiCError_t> {
        let mut available: bool8_t = 0;

        let get_dev_available = unsafe { (*(*self.api_table).IStInterface).IsDeviceAvailable.unwrap() };

        let err = unsafe {
            get_dev_available(
                ptr::addr_of!(self.ptr) as *mut _,
                index,
                access_flag as u32,
                &mut available,
            )
        };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(available != 0)
    }

}

impl Drop for InterfaceHandle {
    fn drop(&mut self) {
        // No explicit release function for interfaces in the API
    }
}

impl InterfaceInfoHandle {
    pub fn get_interface_id(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_iface_id = unsafe { (*(*self.api_table).IStInterfaceInfo).GetIDA.unwrap() };
        let err = unsafe { get_iface_id(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }
 
    pub fn get_interface_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_iface_name = unsafe { (*(*self.api_table).IStInterfaceInfo).GetDisplayNameA.unwrap() };
        let err = unsafe { get_iface_name(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }
 
    pub fn get_interface_type(&self) -> Result<InterfaceType, _EStApiCError_t> {
        todo!("implement: GetInterfaceType")
    }
}

impl Drop for InterfaceInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for interface info in the API
    }
}


// ============================================================================
// Port Handle (IStPort & IStPortInfo)
// ============================================================================

pub struct PortHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}


pub struct PortInfoHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct INodeMapHandle {
    nodemap_ptr: StApiHandle_t,
    api_table: *mut GenApi_Functions_t,
}

impl PortHandle {
    pub fn get_ist_port_info(&self) -> Result<PortInfoHandle, _EStApiCError_t> {
        let mut port_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_port_info = unsafe { (*(*self.api_table).IStPort).GetIStPortInfo.unwrap() };
        let err = unsafe { get_port_info(ptr::addr_of!(self.ptr) as *mut _, &mut port_info_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(PortInfoHandle {
            ptr: port_info_ptr,
            api_table: self.api_table,
        })
    }

    pub fn get_inode_map(&self, genapi_table: *mut GenApi_Functions_t) -> Result<INodeMapHandle, _EStApiCError_t> {
        let mut inode_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_inode_map = unsafe { (*(*self.api_table).IStPort).GetINodeMap.unwrap() };
        let err = unsafe { get_inode_map(ptr::addr_of!(self.ptr) as *mut _, &mut inode_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(INodeMapHandle {
            nodemap_ptr: inode_ptr,
            api_table: genapi_table,
        })
    }
}

impl Drop for PortHandle {
    fn drop(&mut self) {
        // No explicit release function for ports in the API
    }
}

impl PortInfoHandle {
    pub fn get_port_id(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_port_id = unsafe { (*(*self.api_table).IStPortInfo).GetIDA.unwrap() };
        let err = unsafe { get_port_id(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }
    

    pub fn get_port_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_port_name = unsafe { (*(*self.api_table).IStPortInfo).GetPortNameA.unwrap() };
        let err = unsafe { get_port_name(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

}

impl Drop for PortInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for ports in the API
    }
}

// ============================================================================
// Device Handle (IStDevice & IStDeviceInfo)
// ============================================================================

pub struct DeviceHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct DeviceInfoHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct DeviceInfo {
    pub id: String,
    pub display_name: String,
    pub access_status: DeviceAccess,
    pub version: String,
}

impl DeviceHandle {

    pub fn get_local_port(&self) -> Result<PortHandle, _EStApiCError_t> {
        let mut port_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_port = unsafe { (*(*self.api_table).IStDevice).GetLocalIStPort.unwrap() };

        let err = unsafe { get_port(ptr::addr_of!(self.ptr) as *mut _, &mut port_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(PortHandle {
            ptr: port_ptr,
            api_table: self.api_table,
        })
    }

    pub fn get_remote_port(&self) -> Result<PortHandle, _EStApiCError_t> {
        let mut port_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_port = unsafe { (*(*self.api_table).IStDevice).GetRemoteIStPort.unwrap() };

        let err = unsafe { get_port(ptr::addr_of!(self.ptr) as *mut _, &mut port_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(PortHandle {
            ptr: port_ptr,
            api_table: self.api_table,
        })
    }

    pub fn get_device_info(&self) -> Result<DeviceInfoHandle, _EStApiCError_t> {
        let mut dev_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_dev_info = unsafe { (*(*self.api_table).IStDevice).GetIStDeviceInfo.unwrap() };

        let err = unsafe { get_dev_info(ptr::addr_of!(self.ptr) as *mut _, &mut dev_info_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(DeviceInfoHandle {
            ptr: dev_info_ptr,
            api_table: self.api_table,
        })
    }

    pub fn get_datastream_count(&self) -> Result<u32, _EStApiCError_t> {
        let mut count: u32 = 0;

        let get_ds_count = unsafe { (*(*self.api_table).IStDevice).GetDataStreamCount.unwrap() };

        let err = unsafe { get_ds_count(ptr::addr_of!(self.ptr) as *mut _, &mut count) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(count)
    }
    pub fn create_datastream(&self, index: usize) -> Result<DataStreamHandle, _EStApiCError_t> {
        let mut datastream_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let create_ds = unsafe { (*(*self.api_table).IStDevice).CreateIStDataStream.unwrap() };

        let err = unsafe {create_ds(ptr::addr_of!(self.ptr) as *mut _,index,ptr::null_mut(),&mut datastream_ptr)
        };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(DataStreamHandle { 
            ptr: datastream_ptr,
            api_table: self.api_table 
        })
    }
    
    //IStDataStream.StartAcquisition() must be called beforehand to enable data acquisition in the host side. To stop acquisition, AcquisitionStop() must be called.
    pub fn acquisition_start(&self) -> Result<(), _EStApiCError_t> {
        let acq_start = unsafe { (*(*self.api_table).IStDevice).AcquisitionStart.unwrap() };
        let err = unsafe { acq_start(ptr::addr_of!(self.ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    //IStDataStream.StopAcquisition() must be called to stop the data acquisition in the host side.
    pub fn acquisition_stop(&self) -> Result<(), _EStApiCError_t> {
        let acq_stop = unsafe { (*(*self.api_table).IStDevice).AcquisitionStop.unwrap() };
        let err = unsafe { acq_stop(ptr::addr_of!(self.ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn device_lost(&self) -> Result<bool, _EStApiCError_t> {
        let mut is_lost: bool8_t = 0;
        let check_lost = unsafe { (*(*self.api_table).IStDevice).IsDeviceLost.unwrap() };
        let err = unsafe { check_lost(ptr::addr_of!(self.ptr) as *mut _, &mut is_lost) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        //returns true if the device is lost, false otherwise
        Ok(is_lost != 0)
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStDevice).Release {
                release(&mut self.ptr);
            }
        }
    }
}

impl DeviceInfoHandle {
    pub fn get_dev_id(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_dev_id = unsafe { (*(*self.api_table).IStDeviceInfo).GetIDA.unwrap() };
        let err = unsafe { get_dev_id(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_dev_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_dev_name = unsafe { (*(*self.api_table).IStDeviceInfo).GetDisplayNameA.unwrap() };
        let err = unsafe { get_dev_name(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_dev_status(&self) -> Result<DeviceAccess, _EStApiCError_t> {
        let mut access_status: u32 = 0;
        let get_dev_status = unsafe { (*(*self.api_table).IStDeviceInfo).GetAccessStatus.unwrap() };
        let err = unsafe { get_dev_status(ptr::addr_of!(self.ptr) as *mut _, &mut access_status) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        // Convert the returned access status to the DeviceAccess enum
        DeviceAccess::from_repr(access_status)
            .ok_or(_EStApiCError_t_StApiCError_OutOfRange)
    }

    pub fn get_dev_version(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_dev_version = unsafe { (*(*self.api_table).IStDeviceInfo).GetVersionA.unwrap() };
        let err = unsafe { get_dev_version(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }
}

impl Drop for DeviceInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for device info in the API
    }
}

// ============================================================================
// Data Stream Handle (IStDataStream & IStDataStreamInfo)
// ============================================================================

pub struct DataStreamHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct DataStreamInfoHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum DSStartFlag {
    DEFAULT = ACQ_START_FLAGS_ACQ_START_FLAGS_DEFAULT,
    CUSTOM_ID = ACQ_START_FLAGS_ACQ_START_FLAGS_CUSTOM_ID,

}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum DSStopFlag {
    DEFAULT = ACQ_STOP_FLAGS_ACQ_STOP_FLAGS_DEFAULT,
    KILL = ACQ_STOP_FLAGS_ACQ_STOP_FLAGS_KILL,
    CUSTOM_ID = ACQ_STOP_FLAGS_ACQ_STOP_FLAGS_CUSTOM_ID,
}



impl DataStreamHandle {

    pub fn start_event_acquisition(&self) -> Result<(), _EStApiCError_t> {
        let start_acq = unsafe { (*(*self.api_table).IStDataStream).StartEventAcquisitionThread.unwrap() };
        let err = unsafe { start_acq(ptr::addr_of!(self.ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn stop_event_acquisition(&self) -> Result<(), _EStApiCError_t> {
        let stop_acq = unsafe { (*(*self.api_table).IStDataStream).StopEventAcquisitionThread.unwrap() };
        let err = unsafe { stop_acq(ptr::addr_of!(self.ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_datastream_info(&self) -> Result<DataStreamInfoHandle, _EStApiCError_t> {
        let mut ds_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_ds_info = unsafe { (*(*self.api_table).IStDataStream).GetIStDataStreamInfo.unwrap() };

        let err = unsafe { get_ds_info(ptr::addr_of!(self.ptr) as *mut _, &mut ds_info_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(DataStreamInfoHandle {
            ptr: ds_info_ptr,
            api_table: self.api_table,
        })
    }
    //IStDevice::AcquisitionStart() call is required for the device to send the data out. To stop data acquisition, call StopAcquisition().
    pub fn start_acquisition(&self, num_acquisitions: u64, acq_start_flag: u32) -> Result<(), _EStApiCError_t> {
        let start_acq = unsafe { (*(*self.api_table).IStDataStream).StartAcquisition.unwrap() };
        let err = unsafe { start_acq(ptr::addr_of!(self.ptr) as *mut _, num_acquisitions, acq_start_flag) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
    
    //IStDevice::AcquisitionStop() call is required for the device to stop sending the data out.
    pub fn stop_acquisition(&self, acq_stop_flag: u32) -> Result<(), _EStApiCError_t> {
        let stop_acq = unsafe { (*(*self.api_table).IStDataStream).StopAcquisition.unwrap() };
        let err = unsafe { stop_acq(ptr::addr_of!(self.ptr) as *mut _, acq_stop_flag) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    //Note that even if IStDataStreamInfo::IsGrabbing() is false, if IStDataStreamInfo::GetNumAwaitDelivery() is larger than 0 this will still return true.
    pub fn is_grabbing(&self) -> Result<bool, _EStApiCError_t> {
        let mut grabbing: bool8_t = 0;
        let check_grabbing = unsafe { (*(*self.api_table).IStDataStream).IsGrabbing.unwrap() };
        let err = unsafe { check_grabbing(ptr::addr_of!(self.ptr) as *mut _, &mut grabbing) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        //Return true if data acquisition from the device(camera) is on going.
        Ok(grabbing != 0)
    }

    pub fn retrieve_buffer(&self, timeout_ms: u32) -> Result<StreamBufferHandle, _EStApiCError_t> {
        let mut stream_buf_handle: StApiHandle_t = unsafe { mem::zeroed() };
        let retrieve_buffer = unsafe { (*(*self.api_table).IStDataStream).RetrieveBuffer.unwrap() };
        let err = unsafe { retrieve_buffer(ptr::addr_of!(self.ptr) as *mut _, timeout_ms, &mut stream_buf_handle) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        //NULL if timeout happens or the data stream is stopped.
        Ok(StreamBufferHandle { 
            ptr: stream_buf_handle,
            api_table: self.api_table,
         })
    }


}

impl Drop for DataStreamHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStDataStream).Release {
                release(&mut self.ptr);
            }
        }
    }
}

impl DataStreamInfoHandle {
    pub fn get_datastream_id(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_ds_id = unsafe { (*(*self.api_table).IStDataStreamInfo).GetIDA.unwrap() };
        let err = unsafe { get_ds_id(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_payload_size(&self) -> Result<usize, _EStApiCError_t> {
        let mut payload_size: usize = 0;
        let get_payload_size = unsafe { (*(*self.api_table).IStDataStreamInfo).GetPayloadSize.unwrap() };
        let err = unsafe { get_payload_size(ptr::addr_of!(self.ptr) as *mut _, &mut payload_size) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(payload_size)
    }
}

impl Drop for DataStreamInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for data stream info in the API
    }
}

// ===========================================================================
// Stream Buffer (IStStreamBuffer & IStStreamBufferInfo)
// ============================================================================

pub struct StreamBufferHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct StreamBufferInfoHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum PayloadType {
    Unknown = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_UNKNOWN,
    Image = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_IMAGE,
    Raw_Data = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_RAW_DATA,
    File = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_FILE,
    Chunk_Data = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_CHUNK_DATA,
    Jpeg = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_JPEG,
    Jpeg_2000 = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_JPEG2000,
    H264 = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_H264,
    Chunk_Only = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_CHUNK_ONLY,
    Device_Specific = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_DEVICE_SPECIFIC,
    Multi_Part = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_MULTI_PART,
    Custom_Id = PAYLOADTYPE_INFO_IDS_PAYLOAD_TYPE_CUSTOM_ID,
}

impl StreamBufferHandle {
    pub fn start_event_acquisition(&self) -> Result<(), _EStApiCError_t> {
        let start_acq = unsafe { (*(*self.api_table).IStStreamBuffer).StartEventAcquisitionThread.unwrap() };
        let err = unsafe { start_acq(ptr::addr_of!(self.ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn stop_event_acquisition(&self) -> Result<(), _EStApiCError_t> {
        let stop_acq = unsafe { (*(*self.api_table).IStStreamBuffer).StopEventAcquisitionThread.unwrap() };
        let err = unsafe { stop_acq(ptr::addr_of!(self.ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_stream_buffer_info(&self) -> Result<StreamBufferInfoHandle, _EStApiCError_t> {
        let mut sb_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_sb_info = unsafe { (*(*self.api_table).IStStreamBuffer).GetIStStreamBufferInfo.unwrap() };

        let err = unsafe { get_sb_info(ptr::addr_of!(self.ptr) as *mut _, &mut sb_info_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(StreamBufferInfoHandle {
            ptr: sb_info_ptr,
            api_table: self.api_table,
        })
    }
}

impl Drop for StreamBufferHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStStreamBuffer).Release {
                release(&mut self.ptr);
            }
        }
    }
}

impl StreamBufferInfoHandle {

    pub fn get_timestamp(&self) -> Result<u64, _EStApiCError_t> {
        let mut timestamp: u64 = 0;
        let get_timestamp = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetTimestamp.unwrap() };
        let err = unsafe { get_timestamp(ptr::addr_of!(self.ptr) as *mut _, &mut timestamp) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(timestamp)
    }

    pub fn get_new_data(&self) -> Result<bool, _EStApiCError_t> {
        let mut new_data: bool8_t = 0;
        let check_new_data = unsafe { (*(*self.api_table).IStStreamBufferInfo).IsNewData.unwrap() };
        let err = unsafe { check_new_data(ptr::addr_of!(self.ptr) as *mut _, &mut new_data) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(new_data != 0)
    }

    pub fn get_size_filled(&self) -> Result<usize, _EStApiCError_t> {
        let mut size_filled: usize = 0;
        let get_size_filled = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetSizeFilled.unwrap() };
        let err = unsafe { get_size_filled(ptr::addr_of!(self.ptr) as *mut _, &mut size_filled) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(size_filled)
    }

    pub fn get_width(&self) -> Result<usize, _EStApiCError_t> {
        let mut width: usize = 0;
        let get_width = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetWidth.unwrap() };
        let err = unsafe { get_width(ptr::addr_of!(self.ptr) as *mut _, &mut width) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(width)
    }

    pub fn get_height(&self) -> Result<usize, _EStApiCError_t> {
        let mut height: usize = 0;
        let get_height = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetHeight.unwrap() };
        let err = unsafe { get_height(ptr::addr_of!(self.ptr) as *mut _, &mut height) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(height)
    }

    pub fn x_offset(&self) -> Result<usize, _EStApiCError_t> {
        let mut x_offset: usize = 0;
        let get_x_offset = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetXOffset.unwrap() };
        let err = unsafe { get_x_offset(ptr::addr_of!(self.ptr) as *mut _, &mut x_offset) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(x_offset)
    }

    pub fn y_offset(&self) -> Result<usize, _EStApiCError_t> {
        let mut y_offset: usize = 0;
        let get_y_offset = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetYOffset.unwrap() };
        let err = unsafe { get_y_offset(ptr::addr_of!(self.ptr) as *mut _, &mut y_offset) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(y_offset)
    }

    pub fn x_padding(&self) -> Result<usize, _EStApiCError_t> {
        let mut x_padding: usize = 0;
        let get_x_padding = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetXPadding.unwrap() };
        let err = unsafe { get_x_padding(ptr::addr_of!(self.ptr) as *mut _, &mut x_padding) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(x_padding)
    }

    pub fn y_padding(&self) -> Result<usize, _EStApiCError_t> {
        let mut y_padding: usize = 0;
        let get_y_padding = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetYPadding.unwrap() };
        let err = unsafe { get_y_padding(ptr::addr_of!(self.ptr) as *mut _, &mut y_padding) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(y_padding)
    }

    pub fn get_frame_id(&self) -> Result<u64, _EStApiCError_t> {
        let mut frame_id: u64 = 0;
        let get_frame_id = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetFrameID.unwrap() };
        let err = unsafe { get_frame_id(ptr::addr_of!(self.ptr) as *mut _, &mut frame_id) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(frame_id)
    }

    pub fn check_image_present(&self) -> Result<bool, _EStApiCError_t> {
        let mut is_present: bool8_t = 0;
        let check_image_present = unsafe { (*(*self.api_table).IStStreamBufferInfo).IsImagePresent.unwrap() };
        let err = unsafe { check_image_present(ptr::addr_of!(self.ptr) as *mut _, &mut is_present) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(is_present != 0)
    }

    pub fn get_image_offset(&self) -> Result<usize, _EStApiCError_t> {
        let mut image_offset: usize = 0;
        let get_image_offset = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetImageOffset.unwrap() };
        let err = unsafe { get_image_offset(ptr::addr_of!(self.ptr) as *mut _, &mut image_offset) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(image_offset)
    }

    pub fn get_payload_type(&self) -> Result<PayloadType, _EStApiCError_t> {
        let mut payload_type: usize = 0;
        let get_payload_type = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetPayloadType.unwrap() };
        let err = unsafe { get_payload_type(ptr::addr_of!(self.ptr) as *mut _, &mut payload_type) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        PayloadType::from_repr(payload_type as u32)
            .ok_or(_EStApiCError_t_StApiCError_OutOfRange)
    }

    pub fn get_pixel_format(&self) -> Result<u64, _EStApiCError_t> {
        let mut pixel_format: u64 = 0;
        let get_pixel_format = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetPixelFormat.unwrap() };
        let err = unsafe { get_pixel_format(ptr::addr_of!(self.ptr) as *mut _, &mut pixel_format) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(pixel_format)
    }

    pub fn delivered_image_height(&self) -> Result<usize, _EStApiCError_t> {
        let mut delivered_image_height: usize = 0;
        let get_delivered_image_height = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetDeliveredImageHeight.unwrap() };
        let err = unsafe { get_delivered_image_height(ptr::addr_of!(self.ptr) as *mut _, &mut delivered_image_height) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(delivered_image_height)
    }

    pub fn delivered_chunk_payload_size(&self) -> Result<usize, _EStApiCError_t> {
        let mut delivered_chunk_payload_size: usize = 0;
        let get_delivered_chunk_payload_size = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetDeliveredChunkPayloadSize.unwrap() };
        let err = unsafe { get_delivered_chunk_payload_size(ptr::addr_of!(self.ptr) as *mut _, &mut delivered_chunk_payload_size) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(delivered_chunk_payload_size)
    }

    pub fn get_file_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_file_name = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetFileNameA.unwrap() };
        let err = unsafe { get_file_name(ptr::addr_of!(self.ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_data_size(&self) -> Result<usize, _EStApiCError_t> {
        let mut data_size: usize = 0;
        let get_data_size = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetDataSize.unwrap() };
        let err = unsafe { get_data_size(ptr::addr_of!(self.ptr) as *mut _, &mut data_size) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(data_size)
    }

    pub fn get_pixel_endianness(&self) -> Result<i32, _EStApiCError_t> {
        let mut pixel_endianness: i32 = 0;
        let get_pixel_endianness = unsafe { (*(*self.api_table).IStStreamBufferInfo).GetPixelEndianness.unwrap() };
        let err = unsafe { get_pixel_endianness(ptr::addr_of!(self.ptr) as *mut _, &mut pixel_endianness) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(pixel_endianness)
    }

    pub fn check_data_larger_buffer(&self) -> Result<bool, _EStApiCError_t> {
        let mut is_larger: bool8_t = 0;
        let check_larger_buffer = unsafe { (*(*self.api_table).IStStreamBufferInfo).IsDataLargerThanBuffer.unwrap() };
        let err = unsafe { check_larger_buffer(ptr::addr_of!(self.ptr) as *mut _, &mut is_larger) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(is_larger != 0)
    }
}

impl Drop for StreamBufferInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for stream buffer info in the API
    }
}

// ===========================================================================
// Image (IStImage, IStImageBuffer, ImageAveragingFilter)
// ============================================================================

pub struct ImageHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct ImageBufferHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct ImageAveragingFilterHandle {
    image_averaging_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum ImagePixelFormat {
    Unknown = EStPixelFormatNamingConvention_t_StPFNC_Unknown,
    Mono1p = EStPixelFormatNamingConvention_t_StPFNC_Mono1p,
    Mono2p = EStPixelFormatNamingConvention_t_StPFNC_Mono2p,
    Mono4p = EStPixelFormatNamingConvention_t_StPFNC_Mono4p,
    Mono8 = EStPixelFormatNamingConvention_t_StPFNC_Mono8,
    Mono10 = EStPixelFormatNamingConvention_t_StPFNC_Mono10,
    Mono10p = EStPixelFormatNamingConvention_t_StPFNC_Mono10p,
    Mono12 = EStPixelFormatNamingConvention_t_StPFNC_Mono12,
    Mono12p = EStPixelFormatNamingConvention_t_StPFNC_Mono12p,
    Mono14 = EStPixelFormatNamingConvention_t_StPFNC_Mono14,
    Mono14p = EStPixelFormatNamingConvention_t_StPFNC_Mono14p,
    Mono16 = EStPixelFormatNamingConvention_t_StPFNC_Mono16,
    Mono32 = EStPixelFormatNamingConvention_t_StPFNC_Mono32,
    BayerBG4p = EStPixelFormatNamingConvention_t_StPFNC_BayerBG4p,
    BayerBG8 = EStPixelFormatNamingConvention_t_StPFNC_BayerBG8,
    BayerBG10 = EStPixelFormatNamingConvention_t_StPFNC_BayerBG10,
    BayerBG10p = EStPixelFormatNamingConvention_t_StPFNC_BayerBG10p,
    BayerBG12 = EStPixelFormatNamingConvention_t_StPFNC_BayerBG12,
    BayerBG12p = EStPixelFormatNamingConvention_t_StPFNC_BayerBG12p,
    BayerBG14 = EStPixelFormatNamingConvention_t_StPFNC_BayerBG14,
    BayerBG14p = EStPixelFormatNamingConvention_t_StPFNC_BayerBG14p,
    BayerBG16 = EStPixelFormatNamingConvention_t_StPFNC_BayerBG16,
    BayerGB4p = EStPixelFormatNamingConvention_t_StPFNC_BayerGB4p,
    BayerGB8 = EStPixelFormatNamingConvention_t_StPFNC_BayerGB8,
    BayerGB10 = EStPixelFormatNamingConvention_t_StPFNC_BayerGB10,
    BayerGB10p = EStPixelFormatNamingConvention_t_StPFNC_BayerGB10p,
    BayerGB12 = EStPixelFormatNamingConvention_t_StPFNC_BayerGB12,
    BayerGB12p = EStPixelFormatNamingConvention_t_StPFNC_BayerGB12p,
    BayerGB14 = EStPixelFormatNamingConvention_t_StPFNC_BayerGB14,
    BayerGB14p = EStPixelFormatNamingConvention_t_StPFNC_BayerGB14p,
    BayerGB16 = EStPixelFormatNamingConvention_t_StPFNC_BayerGB16,
    BayerGR4p = EStPixelFormatNamingConvention_t_StPFNC_BayerGR4p,
    BayerGR8 = EStPixelFormatNamingConvention_t_StPFNC_BayerGR8,
    BayerGR10 = EStPixelFormatNamingConvention_t_StPFNC_BayerGR10,
    BayerGR10p = EStPixelFormatNamingConvention_t_StPFNC_BayerGR10p,
    BayerGR12 = EStPixelFormatNamingConvention_t_StPFNC_BayerGR12,
    BayerGR12p = EStPixelFormatNamingConvention_t_StPFNC_BayerGR12p,
    BayerGR14 = EStPixelFormatNamingConvention_t_StPFNC_BayerGR14,
    BayerGR14p = EStPixelFormatNamingConvention_t_StPFNC_BayerGR14p,
    BayerGR16 = EStPixelFormatNamingConvention_t_StPFNC_BayerGR16,
    BayerRG4p = EStPixelFormatNamingConvention_t_StPFNC_BayerRG4p,
    BayerRG8 = EStPixelFormatNamingConvention_t_StPFNC_BayerRG8,
    BayerRG10 = EStPixelFormatNamingConvention_t_StPFNC_BayerRG10,
    BayerRG10p = EStPixelFormatNamingConvention_t_StPFNC_BayerRG10p,
    BayerRG12 = EStPixelFormatNamingConvention_t_StPFNC_BayerRG12,
    BayerRG12p = EStPixelFormatNamingConvention_t_StPFNC_BayerRG12p,
    BayerRG14 = EStPixelFormatNamingConvention_t_StPFNC_BayerRG14,
    BayerRG14p = EStPixelFormatNamingConvention_t_StPFNC_BayerRG14p,
    BayerRG16 = EStPixelFormatNamingConvention_t_StPFNC_BayerRG16,
    RGBa8 = EStPixelFormatNamingConvention_t_StPFNC_RGBa8,
    RGB8 = EStPixelFormatNamingConvention_t_StPFNC_RGB8,
    RGB10p32 = EStPixelFormatNamingConvention_t_StPFNC_RGB10p32,
    BGRa8 = EStPixelFormatNamingConvention_t_StPFNC_BGRa8,
    BGRa10 = EStPixelFormatNamingConvention_t_StPFNC_BGRa10,
    BGRa12 = EStPixelFormatNamingConvention_t_StPFNC_BGRa12,
    BGRa14 = EStPixelFormatNamingConvention_t_StPFNC_BGRa14,
    BGRa16 = EStPixelFormatNamingConvention_t_StPFNC_BGRa16,
    BGR8 = EStPixelFormatNamingConvention_t_StPFNC_BGR8,
    BGR10 = EStPixelFormatNamingConvention_t_StPFNC_BGR10,
    BGR12 = EStPixelFormatNamingConvention_t_StPFNC_BGR12,
    BGR14 = EStPixelFormatNamingConvention_t_StPFNC_BGR14,
    BGR16 = EStPixelFormatNamingConvention_t_StPFNC_BGR16,
    RGB8_Planar = EStPixelFormatNamingConvention_t_StPFNC_RGB8_Planar,
    RGB10_Planar = EStPixelFormatNamingConvention_t_StPFNC_RGB10_Planar,
    RGB12_Planar = EStPixelFormatNamingConvention_t_StPFNC_RGB12_Planar,
    RGB16_Planar = EStPixelFormatNamingConvention_t_StPFNC_RGB16_Planar,
    BGR10p = EStPixelFormatNamingConvention_t_StPFNC_BGR10p,
    BGR12p = EStPixelFormatNamingConvention_t_StPFNC_BGR12p,
    RGB10p = EStPixelFormatNamingConvention_t_StPFNC_RGB10p,
    RGB12p = EStPixelFormatNamingConvention_t_StPFNC_RGB12p,
    RGB16 = EStPixelFormatNamingConvention_t_StPFNC_RGB16,
    CBGR10p32 = EStPixelFormatNamingConvention_t_StPFNC_CBGR10p32,
    YCbCr8 = EStPixelFormatNamingConvention_t_StPFNC_YCbCr8,
    YCbCr411_8 = EStPixelFormatNamingConvention_t_StPFNC_YCbCr411_8,
    YCbCr422_8 = EStPixelFormatNamingConvention_t_StPFNC_YCbCr422_8,
    YCbCr709_422_8 = EStPixelFormatNamingConvention_t_StPFNC_YCbCr709_422_8,
    YCbCr601_422_8 = EStPixelFormatNamingConvention_t_StPFNC_YCbCr601_422_8,
    YCbCr411_8_CbYYCrYY = EStPixelFormatNamingConvention_t_StPFNC_YCbCr411_8_CbYYCrYY,
    YCbCr601_411_8_CbYYCrYY = EStPixelFormatNamingConvention_t_StPFNC_YCbCr601_411_8_CbYYCrYY,
    YCbCr709_411_8_CbYYCrYY = EStPixelFormatNamingConvention_t_StPFNC_YCbCr709_411_8_CbYYCrYY,
    YCbCr422_8_CbYCrY = EStPixelFormatNamingConvention_t_StPFNC_YCbCr422_8_CbYCrY,
    YCbCr601_422_8_CbYCrY = EStPixelFormatNamingConvention_t_StPFNC_YCbCr601_422_8_CbYCrY,
    YCbCr709_422_8_CbYCrY = EStPixelFormatNamingConvention_t_StPFNC_YCbCr709_422_8_CbYCrY,
    YCbCr8_CbYCr = EStPixelFormatNamingConvention_t_StPFNC_YCbCr8_CbYCr,
    YCbCr601_8_CbYCr = EStPixelFormatNamingConvention_t_StPFNC_YCbCr601_8_CbYCr,
    YCbCr709_8_CbYCr = EStPixelFormatNamingConvention_t_StPFNC_YCbCr709_8_CbYCr,
    YUV411_8_UYYVYY = EStPixelFormatNamingConvention_t_StPFNC_YUV411_8_UYYVYY,
    YUV422_8_UYVY = EStPixelFormatNamingConvention_t_StPFNC_YUV422_8_UYVY,
    YUV8_UYV = EStPixelFormatNamingConvention_t_StPFNC_YUV8_UYV,
    YUV422_8 = EStPixelFormatNamingConvention_t_StPFNC_YUV422_8,
    Mono10Packed = EStPixelFormatNamingConvention_t_StPFNC_Mono10Packed,
    Mono12Packed = EStPixelFormatNamingConvention_t_StPFNC_Mono12Packed,
    BayerBG10Packed = EStPixelFormatNamingConvention_t_StPFNC_BayerBG10Packed,
    BayerBG12Packed = EStPixelFormatNamingConvention_t_StPFNC_BayerBG12Packed,
    BayerGB10Packed = EStPixelFormatNamingConvention_t_StPFNC_BayerGB10Packed,
    BayerGB12Packed = EStPixelFormatNamingConvention_t_StPFNC_BayerGB12Packed,
    BayerGR10Packed = EStPixelFormatNamingConvention_t_StPFNC_BayerGR10Packed,
    BayerGR12Packed = EStPixelFormatNamingConvention_t_StPFNC_BayerGR12Packed,
    BayerRG10Packed = EStPixelFormatNamingConvention_t_StPFNC_BayerRG10Packed,
    BayerRG12Packed = EStPixelFormatNamingConvention_t_StPFNC_BayerRG12Packed,
    Data8 = EStPixelFormatNamingConvention_t_StPFNC_Data8,
    Data8s = EStPixelFormatNamingConvention_t_StPFNC_Data8s,
    Data16 = EStPixelFormatNamingConvention_t_StPFNC_Data16,
    Data16s = EStPixelFormatNamingConvention_t_StPFNC_Data16s,
    Data32 = EStPixelFormatNamingConvention_t_StPFNC_Data32,
    Data32f = EStPixelFormatNamingConvention_t_StPFNC_Data32f,
    Data32s = EStPixelFormatNamingConvention_t_StPFNC_Data32s,
    Data64 = EStPixelFormatNamingConvention_t_StPFNC_Data64,
    Data64f = EStPixelFormatNamingConvention_t_StPFNC_Data64f,
    Data64s = EStPixelFormatNamingConvention_t_StPFNC_Data64s,
    Pol1Mono8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1Mono8,
    Pol1MonoX8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoX8,
    Pol1MonoY8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoY8,
    Pol1MonoXY8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXY8,
    Pol1Mono10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1Mono10,
    Pol1MonoX10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoX10,
    Pol1MonoY10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoY10,
    Pol1MonoXY10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXY10,
    Pol1Mono12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1Mono12,
    Pol1MonoX12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoX12,
    Pol1MonoY12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoY12,
    Pol1MonoXY12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXY12,
    Pol1BayerRG8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRG8,
    Pol1BayerRGX8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGX8,
    Pol1BayerRGY8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGY8,
    Pol1BayerRGXY8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXY8,
    Pol1BayerRG10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRG10,
    Pol1BayerRGX10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGX10,
    Pol1BayerRGY10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGY10,
    Pol1BayerRGXY10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXY10,
    Pol1BayerRG12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRG12,
    Pol1BayerRGX12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGX12,
    Pol1BayerRGY12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGY12,
    Pol1BayerRGXY12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXY12,
    Pol1Mono10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1Mono10p,
    Pol1MonoX10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoX10p,
    Pol1MonoY10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoY10p,
    Pol1MonoXY10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXY10p,
    Pol1Mono12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1Mono12p,
    Pol1MonoX12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoX12p,
    Pol1MonoY12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoY12p,
    Pol1MonoXY12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXY12p,
    Pol1BayerRG10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRG10p,
    Pol1BayerRGX10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGX10p,
    Pol1BayerRGY10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGY10p,
    Pol1BayerRGXY10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXY10p,
    Pol1BayerRG12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRG12p,
    Pol1BayerRGX12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGX12p,
    Pol1BayerRGY12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGY12p,
    Pol1BayerRGXY12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXY12p,
    Pol1MonoC8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoC8,
    Pol1MonoXC8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXC8,
    Pol1MonoYC8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoYC8,
    Pol1MonoXYC8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXYC8,
    Pol1MonoC10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoC10,
    Pol1MonoXC10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXC10,
    Pol1MonoYC10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoYC10,
    Pol1MonoXYC10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXYC10,
    Pol1MonoC12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoC12,
    Pol1MonoXC12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXC12,
    Pol1MonoYC12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoYC12,
    Pol1MonoXYC12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXYC12,
    Pol1BayerRGC8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGC8,
    Pol1BayerRGXC8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXC8,
    Pol1BayerRGYC8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGYC8,
    Pol1BayerRGXYC8 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXYC8,
    Pol1BayerRGC10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGC10,
    Pol1BayerRGXC10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXC10,
    Pol1BayerRGYC10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGYC10,
    Pol1BayerRGXYC10 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXYC10,
    Pol1BayerRGC12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGC12,
    Pol1BayerRGXC12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXC12,
    Pol1BayerRGYC12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGYC12,
    Pol1BayerRGXYC12 = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXYC12,
    Pol1MonoC10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoC10p,
    Pol1MonoXC10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXC10p,
    Pol1MonoYC10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoYC10p,
    Pol1MonoXYC10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXYC10p,
    Pol1MonoC12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoC12p,
    Pol1MonoXC12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXC12p,
    Pol1MonoYC12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoYC12p,
    Pol1MonoXYC12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1MonoXYC12p,
    Pol1BayerRGC10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGC10p,
    Pol1BayerRGXC10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXC10p,
    Pol1BayerRGYC10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGYC10p,
    Pol1BayerRGXYC10p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXYC10p,
    Pol1BayerRGC12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGC12p,
    Pol1BayerRGXC12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXC12p,
    Pol1BayerRGYC12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGYC12p,
    Pol1BayerRGXYC12p = EStPixelFormatNamingConvention_t_StPFNC_Pol1BayerRGXYC12p,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum InitializeMemory {
    DoNothing = _EStMemoryInitialization_t_StMemoryInitialization_DoNothing,
    FilledWith0 = _EStMemoryInitialization_t_StMemoryInitialization_FilledWith0,
    FilledWith1 = _EStMemoryInitialization_t_StMemoryInitialization_FilledWith1,
    Chart1 = _EStMemoryInitialization_t_StMemoryInitialization_Chart_1,
    Count = _EStMemoryInitialization_t_StMemoryInitialization_Count,
}

impl ImageHandle {
    
    pub fn get_image_width(&self) -> Result<usize, _EStApiCError_t> {
        let mut width: usize = 0;
        let get_width = unsafe { (*(*self.api_table).IStImage).GetImageWidth.unwrap() };
        let err = unsafe { get_width(ptr::addr_of!(self.ptr) as *mut _, &mut width) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(width)
    }

    pub fn get_image_height(&self) -> Result<usize, _EStApiCError_t> {
        let mut height: usize = 0;
        let get_height = unsafe { (*(*self.api_table).IStImage).GetImageHeight.unwrap() };
        let err = unsafe { get_height(ptr::addr_of!(self.ptr) as *mut _, &mut height) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(height)
    }

    pub fn get_image_pixel_format(&self) -> Result<ImagePixelFormat, _EStApiCError_t> {
        let mut pixel_format: u32 = 0;
        let get_pixel_format = unsafe { (*(*self.api_table).IStImage).GetImagePixelFormat.unwrap() };
        let err = unsafe { get_pixel_format(ptr::addr_of!(self.ptr) as *mut _, &mut pixel_format) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        ImagePixelFormat::from_repr(pixel_format)
            .ok_or(_EStApiCError_t_StApiCError_OutOfRange)
    }

    pub fn get_image_buffer(&self) -> Result<*mut c_void, _EStApiCError_t> {
        let mut buffer_ptr: *mut c_void = ptr::null_mut();
        let get_image_buffer = unsafe { (*(*self.api_table).IStImage).GetImageBuffer.unwrap() };
        let err = unsafe { get_image_buffer(ptr::addr_of!(self.ptr) as *mut _, &mut buffer_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(buffer_ptr)
    }

    pub fn get_pixel_component_val(&self, x: usize, y: usize) -> Result<PixelComponentValueHandle, _EStApiCError_t> {
        let mut value_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_pixel_component_val = unsafe { (*(*self.api_table).IStImage).GetIStPixelComponentValue.unwrap() };
        let err = unsafe { get_pixel_component_val(ptr::addr_of!(self.ptr) as *mut _, x, y, &mut value_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(PixelComponentValueHandle { 
            component_val_ptr: value_ptr, 
            api_table: self.api_table 
        })
    }

    pub fn set_image_clipboard(&self) -> Result<(), _EStApiCError_t> {
        let set_clipboard = unsafe { (*(*self.api_table).IStImage).SetIStImageToClipboard.unwrap() };
        let err = unsafe { set_clipboard(ptr::addr_of!(self.ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl ImageBufferHandle {
    pub fn create_image_buffer(&self, allocator: Option<PStApiHandle_t>) -> Result<ImageBufferHandle, _EStApiCError_t> {
        let allocator_handle = allocator.unwrap_or(ptr::null_mut());
        let mut image_buffer_handle: StApiHandle_t = unsafe { mem::zeroed() };
        let create_image_buffer = unsafe { (*(*self.api_table).IStImageBuffer).CreateIStImageBuffer.unwrap() };
        let err = unsafe { create_image_buffer(allocator_handle, &mut image_buffer_handle) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(ImageBufferHandle { 
            ptr: image_buffer_handle, 
            api_table: self.api_table 
        })
    }

    pub fn create_buffer(&self, width: usize, height: usize, pixel_format: ImagePixelFormat, initialize_memory: InitializeMemory) -> Result<(), _EStApiCError_t> {
        let create_buffer = unsafe { (*(*self.api_table).IStImageBuffer).CreateBuffer.unwrap() };
        let err = unsafe { create_buffer(ptr::addr_of!(self.ptr) as *mut _, width, height, pixel_format as u32, initialize_memory as u32) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_image(&self) -> Result<ImageHandle, _EStApiCError_t> {
        let mut image_ptr: StApiHandle_t = unsafe { mem::zeroed() };

        let get_image = unsafe { (*(*self.api_table).IStImageBuffer).GetIStImage.unwrap() };

        let err = unsafe { get_image(ptr::addr_of!(self.ptr) as *mut _, &mut image_ptr) };

        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }

        Ok(ImageHandle {
            ptr: image_ptr,
            api_table: self.api_table,
        })
    }

    pub fn copy_image(&self, image_handle: &ImageHandle) -> Result<(), _EStApiCError_t> {
        let copy_image = unsafe { (*(*self.api_table).IStImageBuffer).CopyImage.unwrap() };
        let err = unsafe { copy_image(ptr::addr_of!(self.ptr) as *mut _, ptr::addr_of!(image_handle.ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl ImageAveragingFilterHandle {
    pub fn get_image_averaging_filter(&self, source_handle: InterfaceHandle) -> Result<ImageAveragingFilterHandle, _EStApiCError_t> {
        let mut image_averaging_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStImageAveragingFilter).GetIStImageAveragingFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut image_averaging_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(ImageAveragingFilterHandle { 
            image_averaging_ptr: image_averaging_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_averaged_image(&self, image_buffer: ImageBufferHandle, component_bitcount: usize) -> Result<usize, _EStApiCError_t> {
        let get_averaged_image = unsafe { (*(*self.api_table).IStImageAveragingFilter).GetAveragedImage.unwrap() };
        let err = unsafe { get_averaged_image(ptr::addr_of!(self.image_averaging_ptr) as *mut _, ptr::addr_of!(image_buffer.ptr) as *mut _, component_bitcount) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(component_bitcount)
    }

    pub fn clear_image_data(&self) -> Result<(), _EStApiCError_t> {
        let clear_data = unsafe { (*(*self.api_table).IStImageAveragingFilter).ClearImageData.unwrap() };
        let err = unsafe { clear_data(ptr::addr_of!(self.image_averaging_ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_frame_count(&self) -> Result<usize, _EStApiCError_t> {
        let mut frame_count: usize = 0;
        let get_frame_count = unsafe { (*(*self.api_table).IStImageAveragingFilter).GetFrameCount.unwrap() };
        let err = unsafe { get_frame_count(ptr::addr_of!(self.image_averaging_ptr) as *mut _, &mut frame_count) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(frame_count)
    }
}

impl Drop for ImageHandle {
    fn drop(&mut self) {
        // No explicit release function for IStImage in the API
    }
}

impl Drop for ImageBufferHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStImageBuffer).Release {
                release(&mut self.ptr);
            }
        }
    }
}

impl Drop for ImageAveragingFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for IStImageAveragingFilter in the API
    }
}


// ===========================================================================
// PixelFormatInfo, PixelComponentValueHandle, PixelComponentInfo, PixelFormatConverter
// ============================================================================

pub struct PixelFormatInfoHandle {
    pixel_format_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct PixelComponentInfoHandle {
    component_info_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct PixelComponentValueHandle {
    component_val_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

impl PixelFormatInfoHandle {
    
    pub fn get_pixel_format_info(&self, pixel_format: ImagePixelFormat) -> Result<PixelFormatInfoHandle, _EStApiCError_t> {
        let mut pixel_format_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_pixel_format_info = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetIStPixelFormatInfo.unwrap() };
        let err = unsafe { get_pixel_format_info(pixel_format as u32, &mut pixel_format_info_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(PixelFormatInfoHandle { 
            pixel_format_ptr: pixel_format_info_ptr, 
            api_table: self.api_table 
        })
    }

    pub fn get_value(&self) -> Result<u32, _EStApiCError_t> {
        let mut value: u32 = 0;
        let get_value = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetValue.unwrap() };
        let err = unsafe { get_value(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(value)
    }

    pub fn get_pixel_format_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_pixel_format_name = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetNameA.unwrap() };
        let err = unsafe { get_pixel_format_name(ptr::addr_of!(self.pixel_format_ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_description(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_description = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetDescriptionA.unwrap() };
        let err = unsafe { get_description(ptr::addr_of!(self.pixel_format_ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_pixel_total_bitcount(&self) -> Result<usize, _EStApiCError_t> {
        let mut bitcount: usize = 0;
        let get_bitcount = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetEachPixelTotalBitCount.unwrap() };
        let err = unsafe { get_bitcount(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut bitcount) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(bitcount)
    }

    pub fn get_component_total_bitcount(&self) -> Result<usize, _EStApiCError_t> {
        let mut bitcount: usize = 0;
        let get_bitcount = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetEachComponentTotalBitCount.unwrap() };
        let err = unsafe { get_bitcount(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut bitcount) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(bitcount)
    }

    pub fn get_component_valid_bitcount(&self) -> Result<usize, _EStApiCError_t> {
        let mut bitcount: usize = 0;
        let get_bitcount = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetEachComponentValidBitCount.unwrap() };
        let err = unsafe { get_bitcount(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut bitcount) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(bitcount)
    }

    pub fn get_pixel_component_count(&self) -> Result<usize, _EStApiCError_t> {
        let mut count: usize = 0;
        let get_count = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetEachPixelTotalComponentCount.unwrap() };
        let err = unsafe { get_count(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut count) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(count)
    }

    pub fn get_pixel_increment_x(&self) -> Result<usize, _EStApiCError_t> {
        let mut increment: usize = 0;
        let get_increment = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetPixelIncrementX.unwrap() };
        let err = unsafe { get_increment(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut increment) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(increment)
    }

    pub fn get_pixel_increment_y(&self) -> Result<usize, _EStApiCError_t> {
        let mut increment: usize = 0;
        let get_increment = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetPixelIncrementY.unwrap() };
        let err = unsafe { get_increment(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut increment) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(increment)
    }

    pub fn is_color(&self) -> Result<u8, _EStApiCError_t> {
        let mut is_color: u8 = 0;
        let get_is_color = unsafe { (*(*self.api_table).IStPixelFormatInfo).IsColor.unwrap() };
        let err = unsafe { get_is_color(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut is_color) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(is_color)
    }

    pub fn is_mono(&self) -> Result<u8, _EStApiCError_t> {
        let mut is_mono: u8 = 0;
        let get_is_mono = unsafe { (*(*self.api_table).IStPixelFormatInfo).IsMono.unwrap() };
        let err = unsafe { get_is_mono(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut is_mono) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(is_mono)
    }

    pub fn is_bayer(&self) -> Result<u8, _EStApiCError_t> {
        let mut is_bayer: u8 = 0;
        let get_is_bayer = unsafe { (*(*self.api_table).IStPixelFormatInfo).IsBayer.unwrap() };
        let err = unsafe { get_is_bayer(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut is_bayer) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(is_bayer)
    }

    pub fn is_compressed(&self) -> Result<u8, _EStApiCError_t> {
        let mut is_compressed: u8 = 0;
        let get_is_compressed = unsafe { (*(*self.api_table).IStPixelFormatInfo).IsCompressed.unwrap() };
        let err = unsafe { get_is_compressed(ptr::addr_of!(self.pixel_format_ptr) as *mut _, &mut is_compressed) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(is_compressed)
    }

    pub fn get_pixel_color_filter(&self, x: usize, y: usize) -> Result<u32, _EStApiCError_t> {
        let mut color_filter: u32 = 0;
        let get_color_filter = unsafe { (*(*self.api_table).IStPixelFormatInfo).GetPixelColorFilter.unwrap() };
        let err = unsafe { get_color_filter(ptr::addr_of!(self.pixel_format_ptr) as *mut _, x, y, &mut color_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(color_filter)
    }

}

impl PixelComponentInfoHandle {

    pub fn get_pixel_component_info(&self, pixel_component: u32) -> Result<PixelComponentInfoHandle, _EStApiCError_t> {
        let mut component_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_component_info = unsafe { (*(*self.api_table).IStPixelComponentInfo).GetIStPixelComponentInfo.unwrap() };
        let err = unsafe { get_component_info(pixel_component, &mut component_info_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(PixelComponentInfoHandle { 
            component_info_ptr: component_info_ptr,
            api_table: self.api_table 
        })
    }
    
    pub fn get_value(&self) -> Result<u32, _EStApiCError_t> {
        let mut value: u32 = 0;
        let get_value = unsafe { (*(*self.api_table).IStPixelComponentInfo).GetValue.unwrap() };
        let err = unsafe { get_value(ptr::addr_of!(self.component_info_ptr) as *mut _, &mut value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(value)
    }

    pub fn get_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_name = unsafe { (*(*self.api_table).IStPixelComponentInfo).GetNameA.unwrap() };
        let err = unsafe { get_name(ptr::addr_of!(self.component_info_ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn get_bitcount(&self) -> Result<usize, _EStApiCError_t> {
        let mut bitcount: usize = 0;
        let get_bitcount = unsafe { (*(*self.api_table).IStPixelComponentInfo).GetBitCount.unwrap() };
        let err = unsafe { get_bitcount(ptr::addr_of!(self.component_info_ptr) as *mut _, &mut bitcount) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(bitcount)
    }
}

impl PixelComponentValueHandle {

    pub fn get_count(&self) -> Result<usize, _EStApiCError_t> {
        let mut count: usize = 0;
        let get_count = unsafe { (*(*self.api_table).IStPixelComponentValue).GetCount.unwrap() };
        let err = unsafe { get_count(ptr::addr_of!(self.component_val_ptr) as *mut _, &mut count) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(count)
    }
    
    pub fn get_value(&self, index: usize) -> Result<i64, _EStApiCError_t> {
        let mut value: i64 = 0;
        let get_value = unsafe { (*(*self.api_table).IStPixelComponentValue).GetValue.unwrap() };
        let err = unsafe { get_value(ptr::addr_of!(self.component_val_ptr) as *mut _, index, &mut value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(value)
    }

    pub fn get_pixel_component(&self, index: usize) -> Result<u32, _EStApiCError_t> {
        let mut component_value: u32 = 0;
        let get_component_index = unsafe { (*(*self.api_table).IStPixelComponentValue).GetPixelComponent.unwrap() };
        let err = unsafe { get_component_index(ptr::addr_of!(self.component_val_ptr) as *mut _, index, &mut component_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(component_value)
    }
}

impl Drop for PixelFormatInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for pixel format info in the API
    }
}

impl Drop for PixelComponentInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for pixel component info in the API
    }
}

impl Drop for PixelComponentValueHandle {
    fn drop(&mut self) {
        // No explicit release function for pixel component value in the API
    }
}

// ===========================================================================
// Feature Bag
// ============================================================================

pub struct FeatureBagHandle {
    feature_bag_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

impl FeatureBagHandle {
    pub fn create_feature_bag(&self) -> Result<FeatureBagHandle, _EStApiCError_t> {
        let mut feature_bag_handle: StApiHandle_t = unsafe { mem::zeroed() };
        let create_feature_bag = unsafe { (*(*self.api_table).IStFeatureBag).Create.unwrap() };
        let err = unsafe { create_feature_bag(&mut feature_bag_handle) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FeatureBagHandle { 
            feature_bag_ptr: feature_bag_handle, 
            api_table: self.api_table 
        })
    }

    pub fn store_nodemap(&self, nodemap_handle: &INodeMapHandle, num_entries: i32) -> Result<i64, _EStApiCError_t> {
        let mut value: i64 = 0;
        let store_nodemap = unsafe { (*(*self.api_table).IStFeatureBag).StoreNodeMapToBag.unwrap() };
        let err = unsafe { store_nodemap(ptr::addr_of!(self.feature_bag_ptr) as *mut _, ptr::addr_of!(nodemap_handle.nodemap_ptr) as *mut _, num_entries, &mut value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(value)
    }

    pub fn store_string(&self, setting: &CStr) -> Result<(), _EStApiCError_t> {;
        let store_string = unsafe { (*(*self.api_table).IStFeatureBag).StoreStringToBagA.unwrap() };
        let err = unsafe { store_string(ptr::addr_of!(self.feature_bag_ptr) as *mut _, setting.as_ptr()) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn store_file(&self, file_name: &CStr) -> Result<(), _EStApiCError_t> {
        let store_file = unsafe { (*(*self.api_table).IStFeatureBag).StoreFileToBagA.unwrap() };
        let err = unsafe { store_file(ptr::addr_of!(self.feature_bag_ptr) as *mut _, file_name.as_ptr()) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn load_features(&self, nodemap_handle: &INodeMapHandle, verify: u8) -> Result<u8, _EStApiCError_t> {
        let mut load_error: u8 = 0;
        let load_features = unsafe { (*(*self.api_table).IStFeatureBag).Load.unwrap() };
        let err = unsafe { load_features(ptr::addr_of!(self.feature_bag_ptr) as *mut _, ptr::addr_of!(nodemap_handle.nodemap_ptr) as *mut _, verify, &mut load_error) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(load_error)
    }

    pub fn save_string(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let save_string = unsafe { (*(*self.api_table).IStFeatureBag).SaveToStringA.unwrap() };
        let err = unsafe { save_string(ptr::addr_of!(self.feature_bag_ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

     pub fn save_file(&self, file_name: &CStr) -> Result<(), _EStApiCError_t> {
        let save_file = unsafe { (*(*self.api_table).IStFeatureBag).SaveToFileA.unwrap() };
        let err = unsafe { save_file(ptr::addr_of!(self.feature_bag_ptr) as *mut _, file_name.as_ptr()) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for FeatureBagHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStFeatureBag).Release {
                release(&mut self.feature_bag_ptr);
            }
        }
    }
}

// ===========================================================================
// IStAllocator if needed
// ============================================================================

// ===========================================================================
// ISt Callback if needed
// ============================================================================

// ===========================================================================
// Filter, FilterArray, FilterInfo
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum FilterType {
    GammaCorrection = EStFilterType_t_StFilterType_GammaCorrection,
    ColorTransformation = EStFilterType_t_StFilterType_ColorTransformation,
    EdgeEnhancement = EStFilterType_t_StFilterType_EdgeEnhancement,
    BalanceRatio = EStFilterType_t_StFilterType_BalanceRatio,
    NoiseReduction = EStFilterType_t_StFilterType_NoiseReduction,
    FlatFieldCorrection = EStFilterType_t_StFilterType_FlatFieldCorrection,
    ChromaSuppression = EStFilterType_t_StFilterType_ChromaSuppression,
    SNMeasurement = EStFilterType_t_StFilterType_SNMeasurement,
    GraphData = EStFilterType_t_StFilterType_GraphData,
    ImageAveraging = EStFilterType_t_StFilterType_ImageAveraging,
    DefectivePixelDetection = EStFilterType_t_StFilterType_DefectivePixelDetection,
    Count = EStFilterType_t_StFilterType_Count
}

pub struct FilterHandle {
    filter_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct FilterArrayHandle {
    filter_array_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct FilterInfoHandle {
    filter_info_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

impl FilterHandle {
    pub fn create_filter(&self, filter_type: u32) -> Result<FilterHandle, _EStApiCError_t> {
        let mut filter_handle: StApiHandle_t = unsafe { mem::zeroed() };
        let create_filter = unsafe { (*(*self.api_table).IStFilter).CreateIStFilter.unwrap() };
        let err = unsafe { create_filter(filter_type, &mut filter_handle) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FilterHandle { 
            filter_ptr: filter_handle, 
            api_table: self.api_table 
        })
    }

    pub fn get_filter(&self, source_handle: InterfaceHandle) -> Result<FilterHandle, _EStApiCError_t> {
        let mut filter_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStFilter).GetIStFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut filter_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FilterHandle { 
            filter_ptr: filter_ptr, 
            api_table: self.api_table 
        })
    }

    pub fn get_filter_info(&self) -> Result<FilterInfoHandle, _EStApiCError_t> {
        let mut filter_info_handle: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter_info = unsafe { (*(*self.api_table).IStFilter).GetIStFilterInfo.unwrap() };
        let err = unsafe { get_filter_info(ptr::addr_of!(self.filter_ptr) as *mut _, &mut filter_info_handle) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FilterInfoHandle { 
            filter_info_ptr: filter_info_handle, 
            api_table: self.api_table 
        })
    }

    pub fn filter(&self) -> Result<ImageHandle, _EStApiCError_t> {
        let mut output_image_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let filter = unsafe { (*(*self.api_table).IStFilter).Filter.unwrap() };
        let err = unsafe { filter(ptr::addr_of!(self.filter_ptr) as *mut _, &mut output_image_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(ImageHandle { 
            ptr: output_image_ptr, 
            api_table: self.api_table 
        })
    }

    pub fn get_nodemap(&self, genapi_table: *mut GenApi_Functions_t) -> Result<INodeMapHandle, _EStApiCError_t> {
        let mut nodemap_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_nodemap = unsafe { (*(*self.api_table).IStFilter).GetINodeMap.unwrap() };
        let err = unsafe { get_nodemap(ptr::addr_of!(self.filter_ptr) as *mut _, &mut nodemap_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(INodeMapHandle { 
            nodemap_ptr: nodemap_ptr, 
            api_table: genapi_table 
        })
    }
}

impl FilterArrayHandle {
    pub fn filter(&self, count: usize) -> Result<ImageHandle, _EStApiCError_t> {
        let mut filter_array_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let filter = unsafe { (*(*self.api_table).IStFilterArray).Filter.unwrap() };
        let err = unsafe { filter(ptr::addr_of!(self.filter_array_ptr) as *mut _, count, &mut filter_array_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(ImageHandle { 
            ptr: filter_array_ptr, 
            api_table: self.api_table 
        })
    }

}

impl FilterInfoHandle {
    pub fn get_filter_info(&self, filter_type: u32) -> Result<FilterInfoHandle, _EStApiCError_t> {
        let mut filter_info_handle: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter_info = unsafe { (*(*self.api_table).IStFilterInfo).GetIStFilterInfo.unwrap() };
        let err = unsafe { get_filter_info(filter_type, &mut filter_info_handle) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FilterInfoHandle { 
            filter_info_ptr: filter_info_handle, 
            api_table: self.api_table 
        })
    }

    pub fn filter_type(&self) -> Result<u32, _EStApiCError_t> {
        let mut filter_type: u32 = 0;
        let get_filter_type = unsafe { (*(*self.api_table).IStFilterInfo).GetFilterType.unwrap() };
        let err = unsafe { get_filter_type(ptr::addr_of!(self.filter_info_ptr) as *mut _, &mut filter_type) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(filter_type)
    }

    pub fn get_filter_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_filter_name = unsafe { (*(*self.api_table).IStFilterInfo).GetFilterNameA.unwrap() };
        let err = unsafe { get_filter_name(ptr::addr_of!(self.filter_info_ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }

    pub fn is_supported(&self, pixel_format: u32) -> Result<u8, _EStApiCError_t> {
        let mut supported: u8 = 0;
        let get_is_supported = unsafe { (*(*self.api_table).IStFilterInfo).IsSupported.unwrap() };
        let err = unsafe { get_is_supported(ptr::addr_of!(self.filter_info_ptr) as *mut _, pixel_format, &mut supported) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(supported)
    }
}

impl Drop for FilterHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStFilter).Release {
                release(&mut self.filter_ptr);
            }
        }
    }
}

impl Drop for FilterArrayHandle {
    fn drop(&mut self) {
        // No explicit release function for filter array in the API
    }
}

impl Drop for FilterInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for filter info in the API
    }
}


// ===========================================================================
// GammaCorrectionFilter
// ============================================================================

pub struct GammaCorrectionFilterHandle {
    gamma_correction_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl GammaCorrectionFilterHandle {
    pub fn get_gamma_correction_filter(&self, source_handle: InterfaceHandle) -> Result<GammaCorrectionFilterHandle, _EStApiCError_t> {
        let mut gamma_correction_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStGammaCorrectionFilter).GetIStGammaCorrectionFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut gamma_correction_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(GammaCorrectionFilterHandle { 
            gamma_correction_ptr: gamma_correction_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_gamma_value(&self) -> Result<f64, _EStApiCError_t> {
        let mut gamma_value: f64 = 0.0;
        let get_gamma_value = unsafe { (*(*self.api_table).IStGammaCorrectionFilter).GetGammaValue.unwrap() };
        let err = unsafe { get_gamma_value(ptr::addr_of!(self.gamma_correction_ptr) as *mut _, &mut gamma_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(gamma_value)
    }

    pub fn set_gamma_value(&self, gamma_value: f64) -> Result<(), _EStApiCError_t> {
        let set_gamma_value = unsafe { (*(*self.api_table).IStGammaCorrectionFilter).SetGammaValue.unwrap() };
        let err = unsafe { set_gamma_value(ptr::addr_of!(self.gamma_correction_ptr) as *mut _, gamma_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for GammaCorrectionFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for gamma correction filter in the API
    }
}

// ===========================================================================
// ColorTransformationFilter
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum ColorTransformationvalueSelector {
    Gain_00 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_00,
    Gain_01 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_01,
    Gain_02 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_02,
    Gain_10 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_10,
    Gain_11 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_11,
    Gain_12 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_12,
    Gain_20 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_20,
    Gain_21 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_21,
    Gain_22 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Gain_22,
    Offset_0 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Offset_0,
    Offset_1 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Offset_1,
    Offset_2 = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Offset_2,
    Count = EStColorTransformationValueSelector_t_StColorTransformationValueSelector_Count
}

pub struct ColorTransformationFilterHandle {
    color_transformation_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl ColorTransformationFilterHandle {
    pub fn get_color_transformation_filter(&self, source_handle: InterfaceHandle) -> Result<ColorTransformationFilterHandle, _EStApiCError_t> {
        let mut color_transformation_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStColorTransformationFilter).GetIStColorTransformationFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut color_transformation_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(ColorTransformationFilterHandle { 
            color_transformation_ptr: color_transformation_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_color_transformation_enable(&self) -> Result<u8, _EStApiCError_t> {
        let mut enabled: u8 = 0;
        let get_enable = unsafe { (*(*self.api_table).IStColorTransformationFilter).GetColorTransformationEnable.unwrap() };
        let err = unsafe { get_enable(ptr::addr_of!(self.color_transformation_ptr) as *mut _, &mut enabled) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(enabled)
    }

    pub fn set_color_transformation_enable(&self, enable: u8) -> Result<(), _EStApiCError_t> {
        let set_enable = unsafe { (*(*self.api_table).IStColorTransformationFilter).SetColorTransformationEnable.unwrap() };
        let err = unsafe { set_enable(ptr::addr_of!(self.color_transformation_ptr) as *mut _, enable) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_color_transformation_value(&self, setting_type: u32) -> Result<f64, _EStApiCError_t> {
        let mut color_transformation_value: f64 = 0.0;
        let get_color_transformation_value = unsafe { (*(*self.api_table).IStColorTransformationFilter).GetColorTransformationValue.unwrap() };
        let err = unsafe { get_color_transformation_value(ptr::addr_of!(self.color_transformation_ptr) as *mut _, setting_type, &mut color_transformation_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(color_transformation_value)
    }

    pub fn set_color_transformation_value(&self, setting_type: u32, color_transformation_value: f64) -> Result<(), _EStApiCError_t> {
        let set_color_transformation_value = unsafe { (*(*self.api_table).IStColorTransformationFilter).SetColorTransformationValue.unwrap() };
        let err = unsafe { set_color_transformation_value(ptr::addr_of!(self.color_transformation_ptr) as *mut _, setting_type, color_transformation_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_hue_correction_value(&self) -> Result<f64, _EStApiCError_t> {
        let mut hue_correction_value: f64 = 0.0;
        let get_hue_correction_value = unsafe { (*(*self.api_table).IStColorTransformationFilter).GetHueCorrection.unwrap() };
        let err = unsafe { get_hue_correction_value(ptr::addr_of!(self.color_transformation_ptr) as *mut _, &mut hue_correction_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(hue_correction_value)
    }

    pub fn set_hue_correction_value(&self, hue_correction_value: f64) -> Result<(), _EStApiCError_t> {
        let set_hue_correction_value = unsafe { (*(*self.api_table).IStColorTransformationFilter).SetHueCorrection.unwrap() };
        let err = unsafe { set_hue_correction_value(ptr::addr_of!(self.color_transformation_ptr) as *mut _, hue_correction_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_saturation_correction_value(&self) -> Result<f64, _EStApiCError_t> {
        let mut saturation_correction_value: f64 = 0.0;
        let get_saturation_correction_value = unsafe { (*(*self.api_table).IStColorTransformationFilter).GetSaturationCorrection.unwrap() };
        let err = unsafe { get_saturation_correction_value(ptr::addr_of!(self.color_transformation_ptr) as *mut _, &mut saturation_correction_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(saturation_correction_value)
    }

    pub fn set_saturation_correction_value(&self, saturation_correction_value: f64) -> Result<(), _EStApiCError_t> {
        let set_saturation_correction_value = unsafe { (*(*self.api_table).IStColorTransformationFilter).SetSaturationCorrection.unwrap() };
        let err = unsafe { set_saturation_correction_value(ptr::addr_of!(self.color_transformation_ptr) as *mut _, saturation_correction_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for ColorTransformationFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for color transformation filter in the API
    }
}

// ===========================================================================
// EdgeEnhancementFilter
// ============================================================================

pub struct EdgeEnhancementFilterHandle {
    edge_enhancement_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl EdgeEnhancementFilterHandle {
    pub fn get_edge_enhancement_filter(&self, source_handle: InterfaceHandle) -> Result<EdgeEnhancementFilterHandle, _EStApiCError_t> {
        let mut edge_enhancement_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStEdgeEnhancementFilter).GetIStEdgeEnhancementFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut edge_enhancement_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(EdgeEnhancementFilterHandle { 
            edge_enhancement_ptr: edge_enhancement_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_strength(&self) -> Result<f64, _EStApiCError_t> {
        let mut strength: f64 = 0.0;
        let get_strength = unsafe { (*(*self.api_table).IStEdgeEnhancementFilter).GetStrength.unwrap() };
        let err = unsafe { get_strength(ptr::addr_of!(self.edge_enhancement_ptr) as *mut _, &mut strength) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(strength)
    }

    pub fn set_strength(&self, strength: f64) -> Result<(), _EStApiCError_t> {
        let set_strength = unsafe { (*(*self.api_table).IStEdgeEnhancementFilter).SetStrength.unwrap() };
        let err = unsafe { set_strength(ptr::addr_of!(self.edge_enhancement_ptr) as *mut _, strength) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_threshold(&self) -> Result<u32, _EStApiCError_t> {
        let mut threshold: u32 = 0;
        let get_threshold = unsafe { (*(*self.api_table).IStEdgeEnhancementFilter).GetThresh.unwrap() };
        let err = unsafe { get_threshold(ptr::addr_of!(self.edge_enhancement_ptr) as *mut _, &mut threshold) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(threshold)
    }

    pub fn set_threshold(&self, threshold: u32) -> Result<(), _EStApiCError_t> {
        let set_threshold = unsafe { (*(*self.api_table).IStEdgeEnhancementFilter).SetThresh.unwrap() };
        let err = unsafe { set_threshold(ptr::addr_of!(self.edge_enhancement_ptr) as *mut _, threshold) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for EdgeEnhancementFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for edge enhancement filter in the API
    }
}
// ===========================================================================
// BalanceRatioFilter
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum BalanceSelector {
    Red = EStBalanceRatioSelector_t_StBalanceRatioSelector_Red,
    Green = EStBalanceRatioSelector_t_StBalanceRatioSelector_Green,
    Blue = EStBalanceRatioSelector_t_StBalanceRatioSelector_Blue,
    Y = EStBalanceRatioSelector_t_StBalanceRatioSelector_Y,
    Cb = EStBalanceRatioSelector_t_StBalanceRatioSelector_Cb,
    Cr = EStBalanceRatioSelector_t_StBalanceRatioSelector_Cr,
    Count = EStBalanceRatioSelector_t_StBalanceRatioSelector_Count
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum BalanceWhiteAuto {
    Off = EStBalanceWhiteAuto_t_StBalanceWhiteAuto_Off,
    Once = EStBalanceWhiteAuto_t_StBalanceWhiteAuto_Once,
    Continuous = EStBalanceWhiteAuto_t_StBalanceWhiteAuto_Continuous,
    Count = EStBalanceWhiteAuto_t_StBalanceWhiteAuto_Count
}

pub struct BalanceRatioFilterHandle {
    balance_ratio_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl BalanceRatioFilterHandle {
    pub fn get_balance_ratio_filter(&self, source_handle: InterfaceHandle) -> Result<BalanceRatioFilterHandle, _EStApiCError_t> {
        let mut balance_ratio_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStBalanceRatioFilter).GetIStBalanceRatioFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut balance_ratio_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(BalanceRatioFilterHandle { 
            balance_ratio_ptr: balance_ratio_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_balance_ratio(&self, balance_selector: u32) -> Result<f64, _EStApiCError_t> {
        let mut balance_ratio: f64 = 0.0;
        let get_balance_ratio = unsafe { (*(*self.api_table).IStBalanceRatioFilter).GetBalanceRatio.unwrap() };
        let err = unsafe { get_balance_ratio(ptr::addr_of!(self.balance_ratio_ptr) as *mut _, balance_selector, &mut balance_ratio) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(balance_ratio)
    }

    pub fn set_balance_ratio(&self, balance_selector: u32, balance_ratio: f64) -> Result<(), _EStApiCError_t> {
        let set_balance_ratio = unsafe { (*(*self.api_table).IStBalanceRatioFilter).SetBalanceRatio.unwrap() };
        let err = unsafe { set_balance_ratio(ptr::addr_of!(self.balance_ratio_ptr) as *mut _, balance_selector, balance_ratio) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_offset_before_gain(&self, balance_selector: u32) -> Result<i32, _EStApiCError_t> {
        let mut offset_before_gain: i32 = 0;
        let get_offset_before_gain = unsafe { (*(*self.api_table).IStBalanceRatioFilter).GetOffsetLevelBeforeGain.unwrap() };
        let err = unsafe { get_offset_before_gain(ptr::addr_of!(self.balance_ratio_ptr) as *mut _, balance_selector, &mut offset_before_gain) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(offset_before_gain)
    }

    pub fn set_offset_before_gain(&self, balance_selector: u32, offset_before_gain: i32) -> Result<(), _EStApiCError_t> {
        let set_offset_before_gain = unsafe { (*(*self.api_table).IStBalanceRatioFilter).SetOffsetLevelBeforeGain.unwrap() };
        let err = unsafe { set_offset_before_gain(ptr::addr_of!(self.balance_ratio_ptr) as *mut _, balance_selector, offset_before_gain) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_offset_after_gain(&self, balance_selector: u32) -> Result<i32, _EStApiCError_t> {
        let mut offset_after_gain: i32 = 0;
        let get_offset_after_gain = unsafe { (*(*self.api_table).IStBalanceRatioFilter).GetOffsetLevelAfterGain.unwrap() };
        let err = unsafe { get_offset_after_gain(ptr::addr_of!(self.balance_ratio_ptr) as *mut _, balance_selector, &mut offset_after_gain) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(offset_after_gain)
    }

    pub fn set_offset_after_gain(&self, balance_selector: u32, offset_after_gain: i32) -> Result<(), _EStApiCError_t> {
        let set_offset_after_gain = unsafe { (*(*self.api_table).IStBalanceRatioFilter).SetOffsetLevelAfterGain.unwrap() };
        let err = unsafe { set_offset_after_gain(ptr::addr_of!(self.balance_ratio_ptr) as *mut _, balance_selector, offset_after_gain) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_balance_white_auto(&self) -> Result<u32, _EStApiCError_t> {
        let mut white_auto: u32 = 0;
        let get_white_auto = unsafe { (*(*self.api_table).IStBalanceRatioFilter).GetBalanceWhiteAuto.unwrap() };
        let err = unsafe { get_white_auto(ptr::addr_of!(self.balance_ratio_ptr) as *mut _, &mut white_auto) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(white_auto)
    }

    pub fn set_balance_white_auto(&self, white_auto: u32) -> Result<(), _EStApiCError_t> {
        let set_white_auto = unsafe { (*(*self.api_table).IStBalanceRatioFilter).SetBalanceWhiteAuto.unwrap() };
        let err = unsafe { set_white_auto(ptr::addr_of!(self.balance_ratio_ptr) as *mut _, white_auto) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for BalanceRatioFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for balance ratio filter in the API
    }
}

// ===========================================================================
// NoiseReductionFilter
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum NoiseReductionMode {
    Off = EStNoiseReductionMode_t_StNoiseReductionMode_Off,
    Simple = EStNoiseReductionMode_t_StNoiseReductionMode_Simple,
    SubtractingLightShieldingImage = EStNoiseReductionMode_t_StNoiseReductionMode_SubtractingLightShieldingImage,
    Count = EStNoiseReductionMode_t_StNoiseReductionMode_Count
}

pub struct NoiseReductionFilterHandle {
    noise_reduction_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl NoiseReductionFilterHandle {
    pub fn get_noise_reduction_filter(&self, source_handle: InterfaceHandle) -> Result<NoiseReductionFilterHandle, _EStApiCError_t> {
        let mut noise_reduction_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStNoiseReductionFilter).GetIStNoiseReductionFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut noise_reduction_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(NoiseReductionFilterHandle { 
            noise_reduction_ptr: noise_reduction_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_noise_reduction_mode(&self) -> Result<u32, _EStApiCError_t> {
        let mut mode: u32 = 0;
        let get_mode = unsafe { (*(*self.api_table).IStNoiseReductionFilter).GetNoiseReductionMode.unwrap() };
        let err = unsafe { get_mode(ptr::addr_of!(self.noise_reduction_ptr) as *mut _, &mut mode) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(mode)
    }

    pub fn set_noise_reduction_mode(&self, mode: u32) -> Result<(), _EStApiCError_t> {
        let set_mode = unsafe { (*(*self.api_table).IStNoiseReductionFilter).SetNoiseReductionMode.unwrap() };
        let err = unsafe { set_mode(ptr::addr_of!(self.noise_reduction_ptr) as *mut _, mode) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_calibration_enabled(&self) -> Result<u8, _EStApiCError_t> {
        let mut calibration_enabled: u8 = 0;
        let get_calibration = unsafe { (*(*self.api_table).IStNoiseReductionFilter).GetCalibrationEnable.unwrap() };
        let err = unsafe { get_calibration(ptr::addr_of!(self.noise_reduction_ptr) as *mut _, &mut calibration_enabled) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(calibration_enabled)
    }

    pub fn set_calibration_enabled(&self, calibration_enabled: u8) -> Result<(), _EStApiCError_t> {
        let set_calibration = unsafe { (*(*self.api_table).IStNoiseReductionFilter).SetCalibrationEnable.unwrap() };
        let err = unsafe { set_calibration(ptr::addr_of!(self.noise_reduction_ptr) as *mut _, calibration_enabled) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for NoiseReductionFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for noise reduction filter in the API
    }
}


// ===========================================================================
// FlatFieldCorrectionFilter
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum FlatFieldCorrectionMode {
    Off = EStFlatFieldCorrectionMode_t_StFlatFieldCorrectionMode_Off,
    Multiplication = EStFlatFieldCorrectionMode_t_StFlatFieldCorrectionMode_Multiplication,
    Addition = EStFlatFieldCorrectionMode_t_StFlatFieldCorrectionMode_Addition,
    Count = EStFlatFieldCorrectionMode_t_StFlatFieldCorrectionMode_Count
}

pub struct FlatFieldCorrectionFilterHandle {
    flat_field_correction_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl FlatFieldCorrectionFilterHandle {
    pub fn get_flatfield_correction_filter(&self, source_handle: InterfaceHandle) -> Result<FlatFieldCorrectionFilterHandle, _EStApiCError_t> {
        let mut flat_field_correction_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStFlatFieldCorrectionFilter).GetIStFlatFieldCorrectionFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut flat_field_correction_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FlatFieldCorrectionFilterHandle { 
            flat_field_correction_ptr: flat_field_correction_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_flatfield_correction_mode(&self) -> Result<u32, _EStApiCError_t> {
        let mut mode: u32 = 0;
        let get_mode = unsafe { (*(*self.api_table).IStFlatFieldCorrectionFilter).GetFlatFieldCorrectionMode.unwrap() };
        let err = unsafe { get_mode(ptr::addr_of!(self.flat_field_correction_ptr) as *mut _, &mut mode) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(mode)
    }

    pub fn set_flatfield_correction_mode(&self, mode: u32) -> Result<(), _EStApiCError_t> {
        let set_mode = unsafe { (*(*self.api_table).IStFlatFieldCorrectionFilter).SetFlatFieldCorrectionMode.unwrap() };
        let err = unsafe { set_mode(ptr::addr_of!(self.flat_field_correction_ptr) as *mut _, mode) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_calibration_enable(&self) -> Result<u8, _EStApiCError_t> {
        let mut calibration_enabled: u8 = 0;
        let get_calibration = unsafe { (*(*self.api_table).IStFlatFieldCorrectionFilter).GetCalibrationEnable.unwrap() };
        let err = unsafe { get_calibration(ptr::addr_of!(self.flat_field_correction_ptr) as *mut _, &mut calibration_enabled) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(calibration_enabled)
    }

    pub fn set_calibration_enable(&self, calibration_enabled: u8) -> Result<(), _EStApiCError_t> {
        let set_calibration = unsafe { (*(*self.api_table).IStFlatFieldCorrectionFilter).SetCalibrationEnable.unwrap() };
        let err = unsafe { set_calibration(ptr::addr_of!(self.flat_field_correction_ptr) as *mut _, calibration_enabled) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_calibration_target_value(&self) -> Result<u32, _EStApiCError_t> {
        let mut target_value: u32 = 0;
        let get_target_value = unsafe { (*(*self.api_table).IStFlatFieldCorrectionFilter).GetCalibrationTargetValue.unwrap() };
        let err = unsafe { get_target_value(ptr::addr_of!(self.flat_field_correction_ptr) as *mut _, &mut target_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(target_value)
    }

    pub fn set_calibration_target_value(&self, target_value: u32) -> Result<(), _EStApiCError_t> {
        let set_target_value = unsafe { (*(*self.api_table).IStFlatFieldCorrectionFilter).SetCalibrationTargetValue.unwrap() };
        let err = unsafe { set_target_value(ptr::addr_of!(self.flat_field_correction_ptr) as *mut _, target_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for FlatFieldCorrectionFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for flat field correction filter in the API
    }
}

// ===========================================================================
// ChromaSuppressionFilter
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum ChromaSuppressionPartSelector {
    LowLuminancePart = EStChromaSuppressionPartSelector_t_StChromaSuppressionPartSelector_LowLuminancePart,
    HighLuminancePart = EStChromaSuppressionPartSelector_t_StChromaSuppressionPartSelector_HighLuminancePart,
    Count = EStChromaSuppressionPartSelector_t_StChromaSuppressionPartSelector_Count
}

pub struct ChromaSuppressionFilterHandle {
    chroma_suppression_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl ChromaSuppressionFilterHandle {
    pub fn get_chroma_suppression_filter(&self, source_handle: InterfaceHandle) -> Result<ChromaSuppressionFilterHandle, _EStApiCError_t> {
        let mut chroma_suppression_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStChromaSuppressionFilter).GetIStChromaSuppressionFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut chroma_suppression_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(ChromaSuppressionFilterHandle { 
            chroma_suppression_ptr: chroma_suppression_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_threshold_value(&self, part_selector: u32) -> Result<u32, _EStApiCError_t> {
        let mut threshold: u32 = 0;
        let get_threshold = unsafe { (*(*self.api_table).IStChromaSuppressionFilter).GetThresholdValue.unwrap() };
        let err = unsafe { get_threshold(ptr::addr_of!(self.chroma_suppression_ptr) as *mut _, part_selector, &mut threshold) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(threshold)
    }

    pub fn set_threshold_value(&self, part_selector: u32, threshold: u32) -> Result<(), _EStApiCError_t> {
        let set_threshold = unsafe { (*(*self.api_table).IStChromaSuppressionFilter).SetThresholdValue.unwrap() };
        let err = unsafe { set_threshold(ptr::addr_of!(self.chroma_suppression_ptr) as *mut _, part_selector, threshold) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_suppression_value(&self, part_selector: u32) -> Result<u32, _EStApiCError_t> {
        let mut suppression_value: u32 = 0;
        let get_suppression_value = unsafe { (*(*self.api_table).IStChromaSuppressionFilter).GetSuppressionValue.unwrap() };
        let err = unsafe { get_suppression_value(ptr::addr_of!(self.chroma_suppression_ptr) as *mut _, part_selector, &mut suppression_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(suppression_value)
    }

    pub fn set_suppression_value(&self, part_selector: u32, suppression_value: u32) -> Result<(), _EStApiCError_t> {
        let set_suppression_value = unsafe { (*(*self.api_table).IStChromaSuppressionFilter).SetSuppressionValue.unwrap() };
        let err = unsafe { set_suppression_value(ptr::addr_of!(self.chroma_suppression_ptr) as *mut _, part_selector, suppression_value) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for ChromaSuppressionFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for chroma suppression filter in the API
    }
}

// ===========================================================================
// SNMeasurementFilter & SNMeasurementResult
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum ROIMode {
    WholeImage = EStROIMode_t_StROIMode_WholeImage,
    Manual = EStROIMode_t_StROIMode_Manual,
    CenterOfImage = EStROIMode_t_StROIMode_CenterOfImage,
    Count = EStROIMode_t_StROIMode_Count
}

pub struct SNMeasurementFilterHandle {
    sn_measurement_filter_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

pub struct SNMeasurementResultHandle {
    sn_measurement_result_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl SNMeasurementFilterHandle {
    pub fn get_sn_measurement_filter(&self, source_handle: InterfaceHandle) -> Result<SNMeasurementFilterHandle, _EStApiCError_t> {
        let mut sn_measurement_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStSNMeasurementFilter).GetIStSNMeasurementFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut sn_measurement_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(SNMeasurementFilterHandle { 
            sn_measurement_filter_ptr: sn_measurement_filter, 
            api_table: self.api_table
        })
    }

    pub fn clear_grabbed_image(&self) -> Result<(), _EStApiCError_t> {
        let clear_grabbed_image = unsafe { (*(*self.api_table).IStSNMeasurementFilter).ClearGrabbedImage.unwrap() };
        let err = unsafe { clear_grabbed_image(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_frame_count(&self) -> Result<usize, _EStApiCError_t> {
        let mut frame_count: usize = 0;
        let get_frame_count = unsafe { (*(*self.api_table).IStSNMeasurementFilter).GetFrameCount.unwrap() };
        let err = unsafe { get_frame_count(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, &mut frame_count) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(frame_count)
    }

    pub fn set_frame_count(&self, frame_count: usize) -> Result<(), _EStApiCError_t> {
        let set_frame_count = unsafe { (*(*self.api_table).IStSNMeasurementFilter).SetFrameCount.unwrap() };
        let err = unsafe { set_frame_count(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, frame_count) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_roi_mode(&self) -> Result<u32, _EStApiCError_t> {
        let mut roi_mode: u32 = 0;
        let get_roi_mode = unsafe { (*(*self.api_table).IStSNMeasurementFilter).GetROIMode.unwrap() };
        let err = unsafe { get_roi_mode(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, &mut roi_mode) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(roi_mode)
    }

    pub fn set_roi_mode(&self, roi_mode: u32) -> Result<(), _EStApiCError_t> {
        let set_roi_mode = unsafe { (*(*self.api_table).IStSNMeasurementFilter).SetROIMode.unwrap() };
        let err = unsafe { set_roi_mode(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, roi_mode) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_offset_x(&self) -> Result<usize, _EStApiCError_t> {
        let mut offset_x: usize = 0;
        let get_offset_x = unsafe { (*(*self.api_table).IStSNMeasurementFilter).GetOffsetX.unwrap() };
        let err = unsafe { get_offset_x(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, &mut offset_x) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(offset_x)
    }

    pub fn set_offset_x(&self, offset_x: usize) -> Result<(), _EStApiCError_t> {
        let set_offset_x = unsafe { (*(*self.api_table).IStSNMeasurementFilter).SetOffsetX.unwrap() };
        let err = unsafe { set_offset_x(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, offset_x) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_offset_y(&self) -> Result<usize, _EStApiCError_t> {
        let mut offset_y: usize = 0;
        let get_offset_y = unsafe { (*(*self.api_table).IStSNMeasurementFilter).GetOffsetY.unwrap() };
        let err = unsafe { get_offset_y(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, &mut offset_y) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(offset_y)
    }

    pub fn set_offset_y(&self, offset_y: usize) -> Result<(), _EStApiCError_t> {
        let set_offset_y = unsafe { (*(*self.api_table).IStSNMeasurementFilter).SetOffsetY.unwrap() };
        let err = unsafe { set_offset_y(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, offset_y) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_width(&self) -> Result<usize, _EStApiCError_t> {
        let mut width: usize = 0;
        let get_width = unsafe { (*(*self.api_table).IStSNMeasurementFilter).GetWidth.unwrap() };
        let err = unsafe { get_width(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, &mut width) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(width)
    }

    pub fn set_width(&self, width: usize) -> Result<(), _EStApiCError_t> {
        let set_width = unsafe { (*(*self.api_table).IStSNMeasurementFilter).SetWidth.unwrap() };
        let err = unsafe { set_width(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, width) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_height(&self) -> Result<usize, _EStApiCError_t> {
        let mut height: usize = 0;
        let get_height = unsafe { (*(*self.api_table).IStSNMeasurementFilter).GetHeight.unwrap() };
        let err = unsafe { get_height(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, &mut height) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(height)
    }

    pub fn set_height(&self, height: usize) -> Result<(), _EStApiCError_t> {
        let set_height = unsafe { (*(*self.api_table).IStSNMeasurementFilter).SetHeight.unwrap() };
        let err = unsafe { set_height(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, height) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn retrieve_snmeasurement_result(&self, timeout: u32) -> Result<SNMeasurementResultHandle, _EStApiCError_t> {
        let mut measurement_result_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_result = unsafe { (*(*self.api_table).IStSNMeasurementFilter).RetrieveIStSNMeasurementResult.unwrap() };
        let err = unsafe { get_result(ptr::addr_of!(self.sn_measurement_filter_ptr) as *mut _, timeout, &mut measurement_result_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(SNMeasurementResultHandle {
            sn_measurement_result_ptr: measurement_result_ptr,
            api_table: self.api_table
        })
    }
}

impl SNMeasurementResultHandle {
    pub fn get_component_count(&self) -> Result<usize, _EStApiCError_t> {
        let mut component_count: usize = 0;
        let get_component_count = unsafe { (*(*self.api_table).IStSNMeasurementResult).GetComponentCount.unwrap() };
        let err = unsafe { get_component_count(ptr::addr_of!(self.sn_measurement_result_ptr) as *mut _, &mut component_count) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(component_count)
    }

    pub fn get_snmeasurement_value(&self, component_index: usize) -> Result<(u32, usize, f64, f64, f64), _EStApiCError_t> {
        let mut component_type: u32 = 0;
        let mut pixel_count: usize = 0;
        let mut average_value: f64 = 0.0;
        let mut temporal_std_dev: f64 = 0.0;
        let mut frame_std_dev: f64 = 0.0;
        let get_value = unsafe { (*(*self.api_table).IStSNMeasurementResult).GetSNMeasurementValue.unwrap() };
        let err = unsafe { get_value(ptr::addr_of!(self.sn_measurement_result_ptr) as *mut _, component_index, &mut component_type, &mut pixel_count, &mut average_value, &mut temporal_std_dev, &mut frame_std_dev) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok((component_type, pixel_count, average_value, temporal_std_dev, frame_std_dev))
    }
}

impl Drop for SNMeasurementFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for SN measurement filter in the API
    }
}

impl Drop for SNMeasurementResultHandle {
    fn drop(&mut self) {
        // No explicit release function for SN measurement result in the API
    }
}

// ===========================================================================
// Converter, ConverterInfo & ReverseConverter
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum ConverterType { 
    PixelFormat = EStConverterType_t_StConverterType_PixelFormat,
    Reverse = EStConverterType_t_StConverterType_Reverse,
    Count = EStConverterType_t_StConverterType_Count
}

pub struct ConverterHandle {
    converter_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

pub struct ConverterInfoHandle {
    converter_info_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

pub struct ReverseConverterHandle {
    reverse_converter_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

// ===========================================================================
// DefectivePixelDetectionFilter
// ============================================================================

pub struct DefectivePixelDetectionFilterHandle {
    defective_pixel_detection_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum DefectivePixelDetectionStatus {
    NotRun = _EStDefectivePixelDetectionStatus_t_StDefectivePixelDetectionStatus_NotRun,
    Succeeded = _EStDefectivePixelDetectionStatus_t_StDefectivePixelDetectionStatus_Succeeded,
    TooManyDefectivePixelDetectedFailed = _EStDefectivePixelDetectionStatus_t_StDefectivePixelDetectionStatus_TooManyDefectivePixelDetectedFailed,
    Failed = _EStDefectivePixelDetectionStatus_t_StDefectivePixelDetectionStatus_Failed,
    Count = _EStDefectivePixelDetectionStatus_t_StDefectivePixelDetectionStatus_Count
} 

impl DefectivePixelDetectionFilterHandle {
    pub fn get_defective_pixel_detection_filter(&self, source_handle: InterfaceHandle) -> Result<DefectivePixelDetectionFilterHandle, _EStApiCError_t> {
        let mut defective_pixel_detection_filter: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filter = unsafe { (*(*self.api_table).IStDefectivePixelDetectionFilter).GetIStDefectivePixelDetectionFilter.unwrap() };
        let err = unsafe { get_filter(ptr::addr_of!(source_handle.ptr) as *mut _, &mut defective_pixel_detection_filter) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(DefectivePixelDetectionFilterHandle { 
            defective_pixel_detection_ptr: defective_pixel_detection_filter, 
            api_table: self.api_table
        })
    }

    pub fn get_detection_result(&self, buffer_size: usize) -> Result<(u32, usize, Vec<_SStDefectivePixelInformation_t>), _EStApiCError_t> {
        let mut detection_status: u32 = 0;
        let mut defective_pixel_count: usize = buffer_size;
        let mut defective_pixel_list: Vec<_SStDefectivePixelInformation_t> = Vec::with_capacity(buffer_size);
        let get_detection_result = unsafe { (*(*self.api_table).IStDefectivePixelDetectionFilter).GetDetectionResult.unwrap() };
        let err = unsafe { get_detection_result(ptr::addr_of!(self.defective_pixel_detection_ptr) as *mut _, &mut detection_status, &mut defective_pixel_count, defective_pixel_list.as_mut_ptr()) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok((detection_status, defective_pixel_count, defective_pixel_list))
    }

    pub fn clear_detection_result(&self) -> Result<(), _EStApiCError_t> {
        let clear_detection_result = unsafe { (*(*self.api_table).IStDefectivePixelDetectionFilter).ClearDetectionResult.unwrap() };
        let err = unsafe { clear_detection_result(ptr::addr_of!(self.defective_pixel_detection_ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl Drop for DefectivePixelDetectionFilterHandle {
    fn drop(&mut self) {
        // No explicit release function for defective pixel detection filter in the API
    }
}

// ===========================================================================
// Filer, FilerInfo, StillImageFiler, VideoFiler
// ============================================================================

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum FilerType {
    StillImage = EStFilerType_t_StFilerType_StillImage,
    Video = EStFilerType_t_StFilerType_Video,
    Count = EStFilerType_t_StFilerType_Count
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum StillImageFileFormat {
    StApiRaw = EStStillImageFileFormat_t_StStillImageFileFormat_StApiRaw,
    Bitmap = EStStillImageFileFormat_t_StStillImageFileFormat_Bitmap,
    JPEG = EStStillImageFileFormat_t_StStillImageFileFormat_JPEG,
    TIff = EStStillImageFileFormat_t_StStillImageFileFormat_TIFF,
    PNG = EStStillImageFileFormat_t_StStillImageFileFormat_PNG,
    Count = EStStillImageFileFormat_t_StStillImageFileFormat_Count
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum VideoFileFormat {
    AVI1 = EStVideoFileFormat_t_StVideoFileFormat_AVI1,
    AVI2 = EStVideoFileFormat_t_StVideoFileFormat_AVI2,
    Count = EStVideoFileFormat_t_StVideoFileFormat_Count
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, FromRepr)]
pub enum VideoCompressionType {
    Uncompressed = EStVideoFileCompression_t_StVideoFileCompression_Uncompressed,
    MotionJPEG = EStVideoFileCompression_t_StVideoFileCompression_MotionJPEG,
    Count = EStVideoFileCompression_t_StVideoFileCompression_Count
}

pub struct FilerHandle {
    filer_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

pub struct FilerInfoHandle {
    filer_info_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

pub struct StillImageFilerHandle {
    still_image_filer_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

pub struct VideoFilerHandle {
    video_filer_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t
}

impl FilerHandle { 
    pub fn create_filer(&self, filer_type: u32) -> Result<FilerHandle, _EStApiCError_t> {
        let mut filer_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let create_filer = unsafe { (*(*self.api_table).IStFiler).CreateIStFiler.unwrap() };
        let err = unsafe { create_filer(filer_type, &mut filer_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FilerHandle { 
            filer_ptr: filer_ptr, 
            api_table: self.api_table
        })
    }

    pub fn get_filer(&self, source_handle: InterfaceHandle) -> Result<FilerHandle, _EStApiCError_t> {
        let mut filer_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filer = unsafe { (*(*self.api_table).IStFiler).GetIStFiler.unwrap() };
        let err = unsafe { get_filer(ptr::addr_of!(source_handle.ptr) as *mut _, &mut filer_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FilerHandle { 
            filer_ptr: filer_ptr, 
            api_table: self.api_table
        })
    }

    pub fn get_filer_info(&self) -> Result<FilerInfoHandle, _EStApiCError_t> {
        let mut filer_info_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_filer_info = unsafe { (*(*self.api_table).IStFiler).GetIStFilerInfo.unwrap() };
        let err = unsafe { get_filer_info(ptr::addr_of!(self.filer_ptr) as *mut _, &mut filer_info_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(FilerInfoHandle { 
            filer_info_ptr, 
            api_table: self.api_table
        })
    }
}

impl FilerInfoHandle {
    pub fn get_filer_type(&self) -> Result<u32, _EStApiCError_t> {
        let mut filer_type: u32 = 0;
        let get_filer_type = unsafe { (*(*self.api_table).IStFilerInfo).GetFilerType.unwrap() };
        let err = unsafe { get_filer_type(ptr::addr_of!(self.filer_info_ptr) as *mut _, &mut filer_type) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(filer_type)
    }

    pub fn get_filer_name(&self) -> Result<String, _EStApiCError_t> {
        let mut len: usize = 256;
        let mut buffer = vec![0u8; len];
        let get_filer_name = unsafe { (*(*self.api_table).IStFilerInfo).GetFilerNameA.unwrap() };
        let err = unsafe { get_filer_name(ptr::addr_of!(self.filer_info_ptr) as *mut _, buffer.as_mut_ptr().cast(), &mut len) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        buffer.truncate(len);
        let cstr = CStr::from_bytes_with_nul(&buffer[..len]).unwrap();
        Ok(cstr.to_string_lossy().into_owned())
    }
}

impl StillImageFilerHandle {
    pub fn get_still_image_filer(&self, source_handle: InterfaceHandle) -> Result<StillImageFilerHandle, _EStApiCError_t> {
        let mut still_image_filer_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_still_image_filer = unsafe { (*(*self.api_table).IStStillImageFiler).GetIStStillImageFiler.unwrap() };
        let err = unsafe { get_still_image_filer(ptr::addr_of!(source_handle.ptr) as *mut _, &mut still_image_filer_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(StillImageFilerHandle { 
            still_image_filer_ptr, 
            api_table: self.api_table
        })
    }

    pub fn is_save_supported(&self, pixel_format: u32, image_file_format: u32) -> Result<u8, _EStApiCError_t> {
        let mut supported: u8 = 0;
        let is_save_supported = unsafe { (*(*self.api_table).IStStillImageFiler).IsSaveSupported.unwrap() };
        let err = unsafe { is_save_supported(ptr::addr_of!(self.still_image_filer_ptr) as *mut _, pixel_format, image_file_format, &mut supported) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(supported)
    }

    pub fn is_load_supported(&self, pixel_format: u32, image_file_format: u32) -> Result<u8, _EStApiCError_t> {
        let mut supported: u8 = 0;
        let is_load_supported = unsafe { (*(*self.api_table).IStStillImageFiler).IsLoadSupported.unwrap() };
        let err = unsafe { is_load_supported(ptr::addr_of!(self.still_image_filer_ptr) as *mut _, pixel_format, image_file_format, &mut supported) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(supported)
    }

    pub fn save_image(&self, image_dat: ImageHandle, image_file_format: u32, file_name: *const i8) -> Result<(), _EStApiCError_t> {
        let save_image = unsafe { (*(*self.api_table).IStStillImageFiler).SaveA.unwrap() };
        let err = unsafe { save_image(ptr::addr_of!(self.still_image_filer_ptr) as *mut _, ptr::addr_of!(image_dat.ptr) as *mut _, image_file_format, file_name) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn load_image(&self, image_buffer: ImageBufferHandle, file_name: *const i8) -> Result<(), _EStApiCError_t> {
        let load_image = unsafe { (*(*self.api_table).IStStillImageFiler).LoadA.unwrap() };
        let err = unsafe { load_image(ptr::addr_of!(self.still_image_filer_ptr) as *mut _, ptr::addr_of!(image_buffer.ptr) as *mut _, file_name) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_quality(&self) -> Result<u32, _EStApiCError_t> {
        let mut quality: u32 = 0;
        let get_quality = unsafe { (*(*self.api_table).IStStillImageFiler).GetQuality.unwrap() };
        let err = unsafe { get_quality(ptr::addr_of!(self.still_image_filer_ptr) as *mut _, &mut quality) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(quality)
    }

    pub fn set_quality(&self, quality: u32) -> Result<(), _EStApiCError_t> {
        let set_quality = unsafe { (*(*self.api_table).IStStillImageFiler).SetQuality.unwrap() };
        let err = unsafe { set_quality(ptr::addr_of!(self.still_image_filer_ptr) as *mut _, quality) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
}

impl VideoFilerHandle {
    pub fn get_video_filer(&self, source_handle: InterfaceHandle) -> Result<VideoFilerHandle, _EStApiCError_t> {
        let mut video_filer_ptr: StApiHandle_t = unsafe { mem::zeroed() };
        let get_video_filer = unsafe { (*(*self.api_table).IStVideoFiler).GetIStVideoFiler.unwrap() };
        let err = unsafe { get_video_filer(ptr::addr_of!(source_handle.ptr) as *mut _, &mut video_filer_ptr) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(VideoFilerHandle { 
            video_filer_ptr, 
            api_table: self.api_table
        })
    }

    pub fn get_video_file_format(&self) -> Result<u32, _EStApiCError_t> {
        let mut video_file_format: u32 = 0;
        let get_video_file_format = unsafe { (*(*self.api_table).IStVideoFiler).GetVideoFileFormat.unwrap() };
        let err = unsafe { get_video_file_format(ptr::addr_of!(self.video_filer_ptr) as *mut _, &mut video_file_format) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(video_file_format)
    }

    pub fn set_video_file_format(&self, video_file_format: u32) -> Result<(), _EStApiCError_t> {
        let set_video_file_format = unsafe { (*(*self.api_table).IStVideoFiler).SetVideoFileFormat.unwrap() };
        let err = unsafe { set_video_file_format(ptr::addr_of!(self.video_filer_ptr) as *mut _, video_file_format) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_video_file_compression(&self) -> Result<u32, _EStApiCError_t> {
        let mut video_file_compression: u32 = 0;
        let get_video_file_compression = unsafe { (*(*self.api_table).IStVideoFiler).GetVideoFileCompression.unwrap() };
        let err = unsafe { get_video_file_compression(ptr::addr_of!(self.video_filer_ptr) as *mut _, &mut video_file_compression) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(video_file_compression)
    }

    pub fn set_video_file_compression(&self, video_file_compression: u32) -> Result<(), _EStApiCError_t> {
        let set_video_file_compression = unsafe { (*(*self.api_table).IStVideoFiler).SetVideoFileCompression.unwrap() };
        let err = unsafe { set_video_file_compression(ptr::addr_of!(self.video_filer_ptr) as *mut _, video_file_compression) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_quality(&self) -> Result<u32, _EStApiCError_t> {
        let mut quality: u32 = 0;
        let get_quality = unsafe { (*(*self.api_table).IStVideoFiler).GetQuality.unwrap() };
        let err = unsafe { get_quality(ptr::addr_of!(self.video_filer_ptr) as *mut _, &mut quality) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(quality)
    }

    pub fn set_quality(&self, quality: u32) -> Result<(), _EStApiCError_t> {
        let set_quality = unsafe { (*(*self.api_table).IStVideoFiler).SetQuality.unwrap() };
        let err = unsafe { set_quality(ptr::addr_of!(self.video_filer_ptr) as *mut _, quality) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_reverse_y(&self) -> Result<u8, _EStApiCError_t> {
        let mut reverse_y: u8 = 0;
        let get_reverse_y = unsafe { (*(*self.api_table).IStVideoFiler).GetReverseY.unwrap() };
        let err = unsafe { get_reverse_y(ptr::addr_of!(self.video_filer_ptr) as *mut _, &mut reverse_y) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(reverse_y)
    }

    pub fn set_reverse_y(&self, reverse_y: u8) -> Result<(), _EStApiCError_t> {
        let set_reverse_y = unsafe { (*(*self.api_table).IStVideoFiler).SetReverseY.unwrap() };
        let err = unsafe { set_reverse_y(ptr::addr_of!(self.video_filer_ptr) as *mut _, reverse_y) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn get_fps(&self) -> Result<f64, _EStApiCError_t> {
        let mut fps: f64 = 0.0;
        let get_fps = unsafe { (*(*self.api_table).IStVideoFiler).GetFPS.unwrap() };
        let err = unsafe { get_fps(ptr::addr_of!(self.video_filer_ptr) as *mut _, &mut fps) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(fps)
    }

    pub fn set_fps(&self, fps: f64) -> Result<(), _EStApiCError_t> {
        let set_fps = unsafe { (*(*self.api_table).IStVideoFiler).SetFPS.unwrap() };
        let err = unsafe { set_fps(ptr::addr_of!(self.video_filer_ptr) as *mut _, fps) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn register_filename(&self, file_name: *const i8) -> Result<(), _EStApiCError_t> {
        let register_filename = unsafe { (*(*self.api_table).IStVideoFiler).RegisterFileNameA.unwrap() };
        let err = unsafe { register_filename(ptr::addr_of!(self.video_filer_ptr) as *mut _, file_name) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn register_image(&self, image_data: ImageHandle, frame_number: u32) -> Result<(), _EStApiCError_t> {
        let register_image = unsafe { (*(*self.api_table).IStVideoFiler).RegisterIStImage.unwrap() };
        let err = unsafe { register_image(ptr::addr_of!(self.video_filer_ptr) as *mut _, ptr::addr_of!(image_data.ptr) as *mut _, frame_number) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn is_stopped(&self) -> Result<u8, _EStApiCError_t> {
        let mut stopped: u8 = 0;
        let is_stopped = unsafe { (*(*self.api_table).IStVideoFiler).IsStopped.unwrap() };
        let err = unsafe { is_stopped(ptr::addr_of!(self.video_filer_ptr) as *mut _, &mut stopped) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(stopped)
    }

    pub fn get_max_frame_count(&self) -> Result<usize, _EStApiCError_t> {
        let mut max_frame_count: usize = 0;
        let get_max_frame_count = unsafe { (*(*self.api_table).IStVideoFiler).GetMaximumFrameCountPerFile.unwrap() };
        let err = unsafe { get_max_frame_count(ptr::addr_of!(self.video_filer_ptr) as *mut _, &mut max_frame_count) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(max_frame_count)
    }

    pub fn set_max_frame_count(&self, max_frame_count: usize) -> Result<(), _EStApiCError_t> {
        let set_max_frame_count = unsafe { (*(*self.api_table).IStVideoFiler).SetMaximumFrameCountPerFile.unwrap() };
        let err = unsafe { set_max_frame_count(ptr::addr_of!(self.video_filer_ptr) as *mut _, max_frame_count) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

    pub fn reset(&self) -> Result<(), _EStApiCError_t> {
        let reset = unsafe { (*(*self.api_table).IStVideoFiler).Reset.unwrap() };
        let err = unsafe { reset(ptr::addr_of!(self.video_filer_ptr) as *mut _) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }

}

impl Drop for FilerHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStFiler).Release {
                release(&mut self.ptr);
            }
        }
    }
}

impl Drop for FilerInfoHandle {
    fn drop(&mut self) {
        // No explicit release function for filer info in the API
    }
}

impl Drop for StillImageFilerHandle {
    fn drop(&mut self) {
        // No explicit release function for still image filer in the API
    }
}

impl Drop for VideoFilerHandle {
    fn drop(&mut self) {
        // No explicit release function for video filer in the API
    }
}

// ==========================================================================================================================
// GraphData, GraphDataBuffer, GraphDataBufferResizable, GraphDataBufferList, GraphDataBufferListResizable, GraphDataFilter
// ==========================================================================================================================

// ======================================================================================
// Wnd, WndInfo, DeviceSelectionWnd, ImageDisplayWnd, InodeMapDisplayWnd,GraphDisplayWnd
// ======================================================================================

// ===========================================================================
// DrawingTool
// ============================================================================