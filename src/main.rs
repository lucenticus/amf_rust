#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("bindings.rs");
fn call_dynamic() -> Result<u32, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("amfrt64.dll")?;
        let func: libloading::Symbol<AMFInit_Fn> = lib.get(AMF_INIT_FUNCTION_NAME)?;
        let version: amf_uint64 = 111u64; //FIXME
        let factory: *mut *mut AMFFactory = std::ptr::null_mut();
        Ok(func.unwrap()(version, factory) as u32)
    }
}

fn main() {
    let res = call_dynamic();
    println!("result: {:?}", res);
}
