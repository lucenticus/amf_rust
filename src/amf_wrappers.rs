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

pub fn init_dx11(context: *mut AMFContext) -> AMF_RESULT {
    unsafe {(*context).pVtbl.as_ref().unwrap().InitDX11.unwrap()(
        context,
        std::ptr::null_mut(),
        AMF_DX_VERSION_AMF_DX11_0,
    )}
}

pub fn create_component(
    factory: *mut AMFFactory,
    context: *mut AMFContext,
    component_id: &str,
) -> (AMF_RESULT, *mut AMFComponent) {
    let id = widestring::U16CString::from_str(component_id).unwrap();
    let mut encoder: *mut AMFComponent = std::ptr::null_mut();
    let result = unsafe {(*factory).pVtbl.as_ref().unwrap().CreateComponent.unwrap()(factory, context, id.as_ptr(), &mut encoder)};
    (result, encoder)
}

#[inline]
pub fn AMFVariantAssignSize(p_dest: &mut AMFVariantStruct, value: AMFSize) -> AMF_RESULT {
    let err_ret = AMFVariantInit(p_dest);
    if err_ret == AMF_RESULT_AMF_OK {
        (*p_dest).type_ = AMF_VARIANT_TYPE_AMF_VARIANT_SIZE;
        (*p_dest).__bindgen_anon_1.sizeValue = value;
    }
    err_ret
}

#[inline]
pub fn AMFVariantAssignInt64(p_dest: &mut AMFVariantStruct, value: i64) -> AMF_RESULT {
    let err_ret = AMFVariantInit(p_dest);
    if err_ret == AMF_RESULT_AMF_OK {
        (*p_dest).type_ = AMF_VARIANT_TYPE_AMF_VARIANT_INT64;
        (*p_dest).__bindgen_anon_1.int64Value = value;
    }
    err_ret
}

#[inline]
pub fn AMFVariantInit(p_variant: &mut AMFVariantStruct) -> AMF_RESULT {
    (*p_variant).type_ = AMF_VARIANT_TYPE_AMF_VARIANT_EMPTY;
    AMF_RESULT_AMF_OK
}

#[inline]
pub fn AMFAssignPropertySize(
    res: &mut AMF_RESULT,
    p_this: *mut AMFComponent,
    name: &str,
    val: AMFSize,
) {
    let mut var = AMFVariantStruct{type_:AMF_VARIANT_TYPE_AMF_VARIANT_EMPTY, __bindgen_anon_1:AMFVariantStruct__bindgen_ty_1{sizeValue: AMFSize{width:0,height:0}}};
    AMFVariantAssignSize(&mut var, val);
    let property_name = U16CString::from_str(name).unwrap();
    unsafe{
        *res = (*p_this).pVtbl.as_ref().unwrap().SetProperty.unwrap()(p_this, property_name.as_ptr(), var);
    }
}

#[inline]
pub fn AMFAssignPropertyInt64(
    res: &mut AMF_RESULT,
    p_this: *mut AMFComponent,
    name: &str,
    val: i64,
) {
    let mut var = AMFVariantStruct{type_:AMF_VARIANT_TYPE_AMF_VARIANT_EMPTY, __bindgen_anon_1:AMFVariantStruct__bindgen_ty_1{int64Value: 0}};
    AMFVariantAssignInt64(&mut var, val);
    let property_name = U16CString::from_str(name).unwrap();
    unsafe{
        *res = (*p_this).pVtbl.as_ref().unwrap().SetProperty.unwrap()(p_this, property_name.as_ptr(), var);
    }
}

pub fn init_encoder(
    encoder: *mut AMFComponent,
    format_in: AMF_SURFACE_FORMAT,
    width_in: amf_int32,
    height_in: amf_int32,
) -> AMF_RESULT {
    unsafe{(*encoder).pVtbl.as_ref().unwrap().Init.unwrap()(encoder, format_in, width_in, height_in)}
}
