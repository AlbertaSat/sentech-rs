use std::{ffi, str::Utf8Error};

use crate::{error::StApiError, ffi::_EStApiCError_t};

// Rust str to C raw
pub fn string_from_raw(raw: *const ffi::c_char) -> Result<String, Utf8Error> {
    unsafe { Ok(ffi::CStr::from_ptr(raw).to_str()?.to_string()) }
}

// C raw to Rust str
pub fn raw_from_str(string: &str) -> *const ffi::c_char { string.as_ptr().cast() } 

pub fn stapi_result(err: _EStApiCError_t) -> Result<(), StApiError>{
    match StApiError::from_repr(err) {
        Some(e) => Err(e),
        None => Ok(()),
    }
}
