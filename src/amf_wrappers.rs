
#![allow(warnings)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("bindings.rs");
use widestring::{U16CString};


pub fn amf_trace_enable_writer(writer_name: &str, enable: bool) -> amf_bool {
    let writer_cstring = U16CString::from_str(writer_name).unwrap();
    let enable_int = if enable { 1 } else { 0 };

    unsafe {
        AMFTraceEnableWriter(writer_cstring.as_ptr(), enable_int)
    }
}