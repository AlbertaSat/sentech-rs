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
    ptr: StApiHandle_t,
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
            ptr: inode_ptr,
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
// Image (IStImage & IStImageBuffer)
// ============================================================================

pub struct ImageHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct ImageBufferHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
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

pub enum MemoryInitialization {

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

impl Drop for ImageHandle {
    fn drop(&mut self) {
        // No explicit release function for IStImage in the API
    }
}

impl ImageBufferHandle {
    pub fn create_image_buffer(&self, allocator: Option<PStApiHandle_t>) -> Result<ImageBufferHandle, _EStApiCError_t> {
        let mut allocator_handle = allocator.unwrap_or(ptr::null_mut());
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

impl Drop for ImageBufferHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(release) = (*(*self.api_table).IStImageBuffer).Release {
                release(&mut self.ptr);
            }
        }
    }
}


// ===========================================================================
// PixelFormatInfo, PixelComponentValueHandle, PixelComponentInfo, PixelFormatConverter
// ============================================================================

pub struct PixelFormatInfo {
    pixel_format_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct PixelComponentValueHandle {
    component_val_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

pub struct PixelComponentInfo {
    component_info_ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

// ===========================================================================
// Feature Bag
// ============================================================================

pub struct FeatureBagHandle {
    ptr: StApiHandle_t,
    api_table: *mut StApi_Functions_t,
}

// ===========================================================================
// Filter, FilterArray, FilterInfo
// ============================================================================

// ===========================================================================
// GammaCorrectionFilter
// ============================================================================

// ===========================================================================
// ColorTransformationFilter
// ============================================================================

// ===========================================================================
// EdgeEnhancementFilter
// ============================================================================

// ===========================================================================
// BalanceRatioFilter
// ============================================================================

// ===========================================================================
// NoiseReductionFilter
// ============================================================================

// ===========================================================================
// FlatFieldCorrectionFilter
// ============================================================================

// ===========================================================================
// ImageAveragingFilter
// ============================================================================

// ===========================================================================
// DefectivePixelDetectionFilter
// ============================================================================

// ===========================================================================
// SNMeasurementFilter & SNMeasurementResult
// ============================================================================

// ===========================================================================
// Converter, ConverterInfo & ReverseConverter
// ============================================================================

// ===========================================================================
// Filer, FilerInfo, StillImageFiler, VideoFiler
// ============================================================================

