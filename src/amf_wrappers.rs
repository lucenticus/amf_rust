#![allow(warnings)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::amf_bindings::*;
use widestring::{U16CString};


pub fn amf_trace_enable_writer(writer_name: &str, enable: bool) -> amf_bool {
    let writer_cstring = U16CString::from_str(writer_name).unwrap();
    let enable_int = if enable { 1 } else { 0 };

    unsafe {
        AMFTraceEnableWriter(writer_cstring.as_ptr(), enable_int)
    }
}

pub fn amf_factory_helper_init() -> Result<(), i32> {
    let res = unsafe { AMFFactoryHelper_Init() };
    if res != AMF_RESULT_AMF_OK as i32 {
        return Err(res);
    }
    Ok(())
}

pub fn amf_trace_set_global_level(level: i32) -> i32 {
    unsafe { AMFTraceSetGlobalLevel(level) }
}

pub fn amf_trace_set_writer_level(writer_name: &str, level: i32) -> i32 {
    let writer_cstring = U16CString::from_str(writer_name).unwrap();
    unsafe { AMFTraceSetWriterLevel(writer_cstring.as_ptr(), level) }
}

pub fn get_amf_factory() -> Result<*mut AMFFactory, &'static str> {
    let factory = unsafe { AMFFactoryHelper_GetFactory() };
    if factory.is_null() {
        Err("Failed to get AMF factory")
    } else {
        Ok(factory)
    }
}

pub fn create_amf_context(factory: *mut AMFFactory) -> Result<*mut AMFContext, &'static str> {
    let mut context: *mut AMFContext = std::ptr::null_mut();
    let res = unsafe { (*factory).pVtbl.as_ref().unwrap().CreateContext.unwrap()(factory, &mut context) };

    if res != AMF_RESULT_AMF_OK {
        Err("Failed to create AMF context")
    } else {
        Ok(context)
    }
}