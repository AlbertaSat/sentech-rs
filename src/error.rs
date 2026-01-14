use thiserror::Error;
use strum::FromRepr;
use crate::ffi::*;

pub type StApiResult<T> = Result<T, StApiError>;

#[repr(i32)]
#[derive(Debug, Copy, Clone, Error, FromRepr)]
pub enum StApiError{
    
}