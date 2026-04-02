use super::{error::*, ffi::*, utils::*};
use std::{
    ffi::{self, CStr, CString, c_char, c_double, c_void},
    mem::{self, MaybeUninit},
    ptr,
    os::raw,
};
use strum::FromRepr;

#[derive(Debug, Copy, Clone)]
pub struct StVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct TransportLayerHandle {
    ptr: StApiHandle_t,
}

impl TransportLayerHandle {
    pub unsafe fn from_raw(ptr: StApiHandle_t) -> Self {
        Self { ptr }
    }
    pub fn as_raw(&self) -> StApiHandle_t {
        self.ptr
    }
}