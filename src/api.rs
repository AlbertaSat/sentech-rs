use super::{error::*, ffi::*, utils::*};
use std::{
    ffi::{CStr, CString, c_char},
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
    pub fn start_acquisition(&self, numAcquisitions: u64, acqStartFlag: u32) -> Result<(), _EStApiCError_t> {
        let start_acq = unsafe { (*(*self.api_table).IStDataStream).StartAcquisition.unwrap() };
        let err = unsafe { start_acq(ptr::addr_of!(self.ptr) as *mut _, numAcquisitions, acqStartFlag) };
        if err != _EStApiCError_t_StApiCError_NoError {
            return Err(err);
        }
        Ok(())
    }
    
    //IStDevice::AcquisitionStop() call is required for the device to stop sending the data out.
    pub fn stop_acquisition(&self, acqStopFlag: u32) -> Result<(), _EStApiCError_t> {
        let stop_acq = unsafe { (*(*self.api_table).IStDataStream).StopAcquisition.unwrap() };
        let err = unsafe { stop_acq(ptr::addr_of!(self.ptr) as *mut _, acqStopFlag) };
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