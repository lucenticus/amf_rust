#![allow(warnings)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::amf_bindings::*;
use std::os::raw::c_void;
use widestring::U16CString;

use std::fs::File;
use std::io::Write;
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D, D3D11_BOX};

pub fn amf_trace_enable_writer(writer_name: &str, enable: bool) -> amf_bool {
    let writer_cstring = U16CString::from_str(writer_name).unwrap();
    let enable_int = if enable { 1 } else { 0 };

    unsafe { AMFTraceEnableWriter(writer_cstring.as_ptr(), enable_int) }
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
    let res =
        unsafe { (*factory).pVtbl.as_ref().unwrap().CreateContext.unwrap()(factory, &mut context) };

    if res != AMF_RESULT_AMF_OK {
        Err("Failed to create AMF context")
    } else {
        Ok(context)
    }
}

pub fn init_dx11(context: *mut AMFContext) -> AMF_RESULT {
    unsafe {
        (*context).pVtbl.as_ref().unwrap().InitDX11.unwrap()(
            context,
            std::ptr::null_mut(),
            AMF_DX_VERSION_AMF_DX11_0,
        )
    }
}

pub fn create_component(
    factory: *mut AMFFactory,
    context: *mut AMFContext,
    component_id: &str,
) -> (AMF_RESULT, *mut AMFComponent) {
    let id = widestring::U16CString::from_str(component_id).unwrap();
    let mut encoder: *mut AMFComponent = std::ptr::null_mut();
    let result = unsafe {
        (*factory).pVtbl.as_ref().unwrap().CreateComponent.unwrap()(
            factory,
            context,
            id.as_ptr(),
            &mut encoder,
        )
    };
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
    let mut var = AMFVariantStruct {
        type_: AMF_VARIANT_TYPE_AMF_VARIANT_EMPTY,
        __bindgen_anon_1: AMFVariantStruct__bindgen_ty_1 {
            sizeValue: AMFSize {
                width: 0,
                height: 0,
            },
        },
    };
    AMFVariantAssignSize(&mut var, val);
    let property_name = U16CString::from_str(name).unwrap();
    unsafe {
        *res = (*p_this).pVtbl.as_ref().unwrap().SetProperty.unwrap()(
            p_this,
            property_name.as_ptr(),
            var,
        );
    }
}

#[inline]
pub fn AMFAssignPropertyInt64(
    res: &mut AMF_RESULT,
    p_this: *mut AMFComponent,
    name: &str,
    val: i64,
) {
    let mut var = AMFVariantStruct {
        type_: AMF_VARIANT_TYPE_AMF_VARIANT_EMPTY,
        __bindgen_anon_1: AMFVariantStruct__bindgen_ty_1 { int64Value: 0 },
    };
    AMFVariantAssignInt64(&mut var, val);
    let property_name = U16CString::from_str(name).unwrap();
    unsafe {
        *res = (*p_this).pVtbl.as_ref().unwrap().SetProperty.unwrap()(
            p_this,
            property_name.as_ptr(),
            var,
        );
    }
}

pub fn init_encoder(
    encoder: *mut AMFComponent,
    format_in: AMF_SURFACE_FORMAT,
    width_in: amf_int32,
    height_in: amf_int32,
) -> AMF_RESULT {
    unsafe {
        (*encoder).pVtbl.as_ref().unwrap().Init.unwrap()(encoder, format_in, width_in, height_in)
    }
}

pub fn alloc_surface(
    context: *mut AMFContext,
    memory_type: AMF_MEMORY_TYPE,
    format: AMF_SURFACE_FORMAT,
    width: amf_int32,
    height: amf_int32,
) -> Result<*mut AMFSurface, AMF_RESULT> {
    let mut surface: *mut AMFSurface = std::ptr::null_mut();
    let result = unsafe {
        ((*context).pVtbl.as_ref().unwrap().AllocSurface.unwrap())(
            context,
            memory_type,
            format,
            width,
            height,
            &mut surface,
        )
    };

    match result {
        AMF_RESULT_AMF_OK => Ok(surface),
        _ => Err(result),
    }
}

// Wrapper for SubmitInput
pub fn submit_input(
    encoder: *mut AMFComponent,
    surface: *mut AMFSurface,
) -> Result<(), AMF_RESULT> {
    let result = unsafe {
        ((*encoder).pVtbl.as_ref().unwrap().SubmitInput.unwrap())(encoder, surface as *mut AMFData)
    };

    match result {
        AMF_RESULT_AMF_OK | AMF_RESULT_AMF_INPUT_FULL => Ok(()),
        _ => Err(result),
    }
}

// Wrapper for Release
pub fn release(surface: *mut AMFSurface) {
    unsafe {
        (*surface).pVtbl.as_ref().unwrap().Release.unwrap()(surface);
    }
}

// Wrapper for QueryOutput
pub fn query_output(encoder: *mut AMFComponent) -> Result<*mut AMFData, AMF_RESULT> {
    let mut data: *mut AMFData = std::ptr::null_mut();
    let result =
        unsafe { ((*encoder).pVtbl.as_ref().unwrap().QueryOutput.unwrap())(encoder, &mut data) };

    match result {
        AMF_RESULT_AMF_OK => Ok(data),
        _ => Err(result),
    }
}

pub fn query_interface(data: *mut AMFData, guid: &AMFGuid) -> Result<*mut AMFBuffer, AMF_RESULT> {
    let mut buffer: *mut AMFBuffer = std::ptr::null_mut();
    let result = unsafe {
        ((*data).pVtbl.as_ref().unwrap().QueryInterface.unwrap())(
            data,
            guid,
            &mut buffer as *mut _ as *mut *mut c_void,
        )
    };

    match result {
        AMF_RESULT_AMF_OK => Ok(buffer),
        _ => Err(result),
    }
}

pub fn get_native(buffer: *mut AMFBuffer) -> *const u8 {
    unsafe { ((*buffer).pVtbl.as_ref().unwrap().GetNative.unwrap())(buffer) as *const u8 }
}

pub fn get_size(buffer: *mut AMFBuffer) -> usize {
    unsafe { ((*buffer).pVtbl.as_ref().unwrap().GetSize.unwrap())(buffer) as usize }
}

pub fn release_buffer(buffer: *mut AMFBuffer) {
    unsafe {
        ((*buffer).pVtbl.as_ref().unwrap().Release.unwrap())(buffer);
    }
}

pub fn release_data(data: *mut AMFData) {
    unsafe {
        ((*data).pVtbl.as_ref().unwrap().Release.unwrap())(data);
    }
}
pub fn write_amf_buffer_to_file(
    file: &mut File,
    buffer: *mut AMFBuffer,
) -> Result<(), std::io::Error> {
    let native_buffer = get_native(buffer);
    let buffer_size = get_size(buffer);
    unsafe { file.write_all(std::slice::from_raw_parts(native_buffer, buffer_size)) }
}

pub fn release_surface(surface: *mut AMFSurface) -> AMF_RESULT {
    unsafe { ((*surface).pVtbl.as_ref().unwrap().Release.unwrap())(surface) }
}

pub fn release_encoder(encoder: *mut AMFComponent) -> AMF_RESULT {
    unsafe { ((*encoder).pVtbl.as_ref().unwrap().Release.unwrap())(encoder) }
}

pub fn release_context(context: *mut AMFContext) -> AMF_RESULT {
    unsafe { ((*context).pVtbl.as_ref().unwrap().Release.unwrap())(context) }
}

pub fn terminate_encoder(encoder: *mut AMFComponent) -> AMF_RESULT {
    unsafe { ((*encoder).pVtbl.as_ref().unwrap().Terminate.unwrap())(encoder) }
}

pub fn terminate_context(context: *mut AMFContext) -> AMF_RESULT {
    unsafe { ((*context).pVtbl.as_ref().unwrap().Terminate.unwrap())(context) }
}

pub fn amf_factory_helper_terminate() {
    unsafe {
        AMFFactoryHelper_Terminate();
    }
}

#[inline(always)]
pub fn IID_AMFBuffer() -> AMFGuid {
    AMFGuid {
        data1: 0xb04b7248,
        data2: 0xb6f0,
        data3: 0x4321,
        data41: 0xb6,
        data42: 0x91,
        data43: 0xba,
        data44: 0xa4,
        data45: 0x74,
        data46: 0x0f,
        data47: 0x9f,
        data48: 0xcb,
    }
}

pub fn get_plane_at(surface: *mut AMFSurface, index: usize) -> Result<*mut AMFPlane, &'static str> {
    if surface.is_null() {
        return Err("Surface is null");
    }
    unsafe {
        let plane = (*surface).pVtbl.as_ref().unwrap().GetPlaneAt.unwrap()(surface, index);
        Ok(plane)
    }
}

pub fn get_width(plane: *mut AMFPlane) -> i32 {
    unsafe { (*plane).pVtbl.as_ref().unwrap().GetWidth.unwrap()(plane) }
}

pub fn get_height(plane: *mut AMFPlane) -> i32 {
    unsafe { (*plane).pVtbl.as_ref().unwrap().GetHeight.unwrap()(plane) }
}

pub fn get_h_pitch(plane: *mut AMFPlane) -> i32 {
    unsafe { (*plane).pVtbl.as_ref().unwrap().GetHPitch.unwrap()(plane) }
}

pub fn get_native_plane(plane: *mut AMFPlane) -> *mut u8 {
    unsafe { (*plane).pVtbl.as_ref().unwrap().GetNative.unwrap()(plane) as *mut u8 }
}

pub fn convert_surface(surface: *mut AMFSurface, memoryTypeIn: AMF_MEMORY_TYPE) -> i32 {
    unsafe { (*surface).pVtbl.as_ref().unwrap().Convert.unwrap()(surface, memoryTypeIn) }
}

pub fn get_dx11_device(context: *mut AMFContext) -> *mut ID3D11Device {
    unsafe {
        (*context).pVtbl.as_ref().unwrap().GetDX11Device.unwrap()(
            context,
            AMF_DX_VERSION_AMF_DX11_0,
        ) as *mut ID3D11Device
    }
}

pub fn get_immediate_context(
    device_dx11: *mut ID3D11Device,
) -> *mut ID3D11DeviceContext {
    let mut device_context_dx11: *mut ID3D11DeviceContext = std::ptr::null_mut();
    unsafe {
        (*device_dx11).GetImmediateContext(&mut device_context_dx11);
    }
    device_context_dx11
}

pub fn copy_resource(
    device_context_dx11: *mut ID3D11DeviceContext,
    src: *mut ID3D11Resource,
    dest: *mut ID3D11Resource,
) {
    unsafe {
        (*device_context_dx11).CopyResource(dest, src);
    }
}

pub fn copy_subresource_region(
    device_context_dx11: *mut ID3D11DeviceContext,
    dest: *mut ID3D11Resource,
    dest_subresource: u32,
    dest_x: u32,
    dest_y: u32,
    dest_z: u32,
    src: *mut ID3D11Resource,
    src_subresource: u32,
    src_box: &D3D11_BOX,
) {
    unsafe {
        (*device_context_dx11).CopySubresourceRegion(
            dest,
            dest_subresource,
            dest_x,
            dest_y,
            dest_z,
            src,
            src_subresource,
            src_box,
        );
    }
}

pub fn flush_device_context(device_context_dx11: *mut ID3D11DeviceContext) {
    unsafe {
        (*device_context_dx11).Flush();
    }
}

pub fn release_device_context_dx11(
    device_context_dx11: *mut ID3D11DeviceContext,
) {
    unsafe {
        (*device_context_dx11).Release();
    }
}
