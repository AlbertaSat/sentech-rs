use thiserror::Error;
use strum::FromRepr;
use crate::ffi::*;

pub type StApiResult<T> = Result<T, _EStApiCError_t>;

/*
Notes from Olivia on Jan 14 2026:

I ran cargo build and then ran:
 `rg "_EStApiCError_t_StApiCError" target/debug/build/sentech-rs-* /out/bindings.rs`
to get the list of error codes from the generated bindings file.

The relevant error codes were listed in the terminal output:
3497:pub const _EStApiCError_t_StApiCError_NoError: _EStApiCError_t = 0;
3499:pub const _EStApiCError_t_StApiCError_Error: _EStApiCError_t = -1073741824;
3501:pub const _EStApiCError_t_StApiCError_BadAlloc: _EStApiCError_t = -1073741823;
3503:pub const _EStApiCError_t_StApiCError_InvalidArgument: _EStApiCError_t = -1073741822;
3505:pub const _EStApiCError_t_StApiCError_OutOfRange: _EStApiCError_t = -1073741821;
3507:pub const _EStApiCError_t_StApiCError_Property: _EStApiCError_t = -1073741820;
3509:pub const _EStApiCError_t_StApiCError_Runtime: _EStApiCError_t = -1073741819;
3511:pub const _EStApiCError_t_StApiCError_LogicalError: _EStApiCError_t = -1073741818;
3513:pub const _EStApiCError_t_StApiCError_AccessError: _EStApiCError_t = -1073741817;
3515:pub const _EStApiCError_t_StApiCError_Timeout: _EStApiCError_t = -1073741816;
3517:pub const _EStApiCError_t_StApiCError_DynamicCast: _EStApiCError_t = -1073741815;
3519:pub const _EStApiCError_t_StApiCError_GenTLError: _EStApiCError_t = -1073741814;
3521:pub const _EStApiCError_t_StApiCError_LinuxError: _EStApiCError_t = -1073741813;

*/

#[repr(i32)]
#[derive(Debug, Copy, Clone, Error, FromRepr)]
pub enum StApiError{

    #[error("Error")]
    Error = _EStApiCError_t_StApiCError_Error
    #[error("BadAllocation")]
    BadAlloc = _EStApiCError_t_StApiCError_BadAlloc
    #[error("InvalidArgument")]
    InvalidArgument = _EStApiCError_t_StApiCError_InvalidArgument
    #[error("OutOfRange")]
    OutOfRange = _EStApiCError_t_StApiCError_OutOfRange
    #[error("PropertyError")]
    Property = _EStApiCError_t_StApiCError_Property
    #[error("RuntimeError")]
    Runtime = _EStApiCError_t_StApiCError_Runtime
    #[error("LogicalError")]
    LogicalError = _EStApiCError_t_StApiCError_LogicalError
    #[error("AccessError")]
    AccessError = _EStApiCError_t_StApiCError_AccessError
    #[error("TimeoutError")]
    Timeout = _EStApiCError_t_StApiCError_Timeout
    #[error("DynamicCastError")]
    DynamicCast = _EStApiCError_t_StApiCError_DynamicCast
    #[error("GenTLError")]
    GenTLError = _EStApiCError_t_StApiCError_Gen
    #[error("LinuxError")]
    LinuxError = _EStApiCError_t_StApiCError_LinuxError

}

