use std::{ffi, str::Utf8Error};

use crate::{error::VmbError, ffi::VmbError_t};

// Rust str to C raw
pub fn string_from_raw(raw: *const ffi::c_char) -> Result<String, Utf8Error> {
    unsafe { Ok(ffi::CStr::from_ptr(raw).to_str()?.to_string()) }
}

// C raw to Rust str
pub fn raw_from_str(string: &str) -> *const ffi::c_char { string.as_ptr().cast() } 

pub fn vmb_result(err: VmbError_t) -> Result<(), VmbError>{
    match VmbError::from_repr(err) {
        Some(e) => Err(e),
        None => Ok(()),
    }
}
