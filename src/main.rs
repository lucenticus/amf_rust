#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("bindings.rs");

static memoryTypeIn: AMF_MEMORY_TYPE = AMF_MEMORY_TYPE_AMF_MEMORY_DX11;
static formatIn: AMF_SURFACE_FORMAT = AMF_SURFACE_FORMAT_AMF_SURFACE_NV12;

static widthIn: amf_int32 = 1920;
static heightIn: amf_int32 = 1080;
static frameRateIn: amf_int32 = 30;
static bitRateIn: amf_int64 = 5000000i64;
static rectSize: amf_int32 = 50;
//static frameCount: amf_int32 = 500;

//static xPos: amf_int32 = 0;
//static yPos: amf_int32 = 0;

static mut pColor1: *mut AMFSurface = std::ptr::null_mut();
static mut pColor2: *mut AMFSurface = std::ptr::null_mut();

use widestring::{U16CString};

fn main() {
    unsafe{
        let mut res = AMFFactoryHelper_Init();
        let console = U16CString::from_str("Console").unwrap();
        let debug_output = U16CString::from_str("DebugOutput").unwrap();
        println!("AMFFactoryHelper_Init result: {:?}", res);
        res = AMFTraceSetGlobalLevel((AMF_TRACE_DEBUG as i32).try_into().unwrap());
        println!("AMFTraceSetGlobalLevel result: {:?}", res);
        let mut is_ok = AMFTraceEnableWriter(console.as_ptr(), 1);
        println!("AMFTraceEnableWriter result: {:?}", is_ok);
        is_ok = AMFTraceEnableWriter(debug_output.as_ptr(), 1);
        println!("AMFTraceEnableWriter result: {:?}", is_ok);
        res = AMFTraceSetWriterLevel(console.as_ptr(), (AMF_TRACE_DEBUG as i32).try_into().unwrap());
        println!("AMFTraceSetGlobalLevel result: {:?}", res);

        let factory = AMFFactoryHelper_GetFactory();
        let mut context: *mut AMFContext = std::ptr::null_mut();
        let mut encoder: *mut AMFComponent  = std::ptr::null_mut();
        res =  ((*factory).pVtbl.as_ref().unwrap().CreateContext.unwrap())(factory, &mut context);
        println!("CreateContext result: {:?}", res);
        res = ((*context).pVtbl.as_ref().unwrap().InitDX11.unwrap())(context, std::ptr::null_mut(), AMF_DX_VERSION_AMF_DX11_0); // can be DX11 device
        println!("InitDX11 result: {:?}", res);
        PrepareFillDX11(context);
        let id = U16CString::from_str("AMFVideoEncoderVCE_AVC").unwrap();
        res = (*factory).pVtbl.as_ref().unwrap().CreateComponent.unwrap()(factory, context, id.as_ptr(), &mut encoder);
        println!("CreateComponent result: {:?}", res);

        let size: AMFSize = AMFConstructSize(widthIn, heightIn);
        let framerate: AMFRate = AMFConstructRate(frameRateIn.try_into().unwrap(), 1);
        let usage = "Usage";
        AMFAssignPropertyInt64(&mut res, encoder, usage, AMF_VIDEO_ENCODER_USAGE_ENUM_AMF_VIDEO_ENCODER_USAGE_TRANSCODING as i64);
        println!("AMFAssignPropertyInt64 usage result: {:?}", res);
        let bitrate = "TargetBitrate";
        AMFAssignPropertyInt64(&mut res, encoder, bitrate, bitRateIn);
        println!("AMFAssignPropertyInt64 bitrate result: {:?}", res);
        let frameSize = "FrameSize";
        AMFAssignPropertySize(&mut res, encoder, frameSize, size);
        println!("AMFAssignPropertySize frameSize result: {:?}", res);
        res = (*encoder).pVtbl.as_ref().unwrap().Init.unwrap()(encoder, formatIn, widthIn, heightIn);
        println!("encoder->Init() result: {:?}", res);
    }
}

#[inline]
fn AMFVariantAssignSize(p_dest: &mut AMFVariantStruct, value: AMFSize) -> AMF_RESULT {
    let err_ret = AMFVariantInit(p_dest);
    if err_ret == AMF_RESULT_AMF_OK {
        (*p_dest).type_ = AMF_VARIANT_TYPE_AMF_VARIANT_SIZE;
        (*p_dest).__bindgen_anon_1.sizeValue = value;
    }
    err_ret
}

#[inline]
fn AMFVariantAssignInt64(p_dest: &mut AMFVariantStruct, value: i64) -> AMF_RESULT {
    let err_ret = AMFVariantInit(p_dest);
    if err_ret == AMF_RESULT_AMF_OK {
        (*p_dest).type_ = AMF_VARIANT_TYPE_AMF_VARIANT_INT64;
        (*p_dest).__bindgen_anon_1.int64Value = value;
    }
    err_ret
}

#[inline]
fn AMFVariantInit(p_variant: &mut AMFVariantStruct) -> AMF_RESULT {
    (*p_variant).type_ = AMF_VARIANT_TYPE_AMF_VARIANT_EMPTY;
    AMF_RESULT_AMF_OK
}

fn AMFAssignPropertySize(
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


fn AMFAssignPropertyInt64(
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

fn AMFConstructSize(width: i32, height: i32) -> AMFSize {
    AMFSize { width, height }
}

fn AMFConstructRate(num: u32, den: u32) -> AMFRate {
    AMFRate { num, den }
}

fn PrepareFillDX11(context: *mut AMFContext) {
    let mut res = unsafe { (*context).pVtbl.as_ref().unwrap().AllocSurface.unwrap()(context, AMF_MEMORY_TYPE_AMF_MEMORY_HOST, formatIn, widthIn, heightIn, &mut pColor1) };
    println!("AllocSurface result: {:?}", res);
    res = unsafe { (*context).pVtbl.as_ref().unwrap().AllocSurface.unwrap()(context, AMF_MEMORY_TYPE_AMF_MEMORY_HOST, formatIn, rectSize, rectSize, &mut pColor2) };
    println!("AllocSurface result: {:?}", res);
    unsafe {
        FillNV12SurfaceWithColor(pColor2, 128, 0, 128);
        FillNV12SurfaceWithColor(pColor1, 128, 255, 128);
        (*pColor1).pVtbl.as_ref().unwrap().Convert.unwrap()(pColor1, memoryTypeIn);
        (*pColor2).pVtbl.as_ref().unwrap().Convert.unwrap()(pColor2, memoryTypeIn);
    }
}

fn FillNV12SurfaceWithColor(surface: *mut AMFSurface, Y: u8, U: u8, V: u8) {
    unsafe {
        let plane_y = (*surface).pVtbl.as_ref().unwrap().GetPlaneAt.unwrap()(surface, 0);
        let plane_uv = (*surface).pVtbl.as_ref().unwrap().GetPlaneAt.unwrap()(surface, 1);

        let width_y = (*plane_y).pVtbl.as_ref().unwrap().GetWidth.unwrap()(plane_y);
        let height_y = (*plane_y).pVtbl.as_ref().unwrap().GetHeight.unwrap()(plane_y);
        let line_y = (*plane_y).pVtbl.as_ref().unwrap().GetHPitch.unwrap()(plane_y);

        let data_y = (*plane_y).pVtbl.as_ref().unwrap().GetNative.unwrap()(plane_y) as *mut u8;

        for y in 0..height_y {
            let data_line = data_y.add(y as usize * line_y as usize);
            std::ptr::write_bytes(data_line, Y, width_y as usize);
        }

        let width_uv = (*plane_uv).pVtbl.as_ref().unwrap().GetWidth.unwrap()(plane_uv);
        let height_uv = (*plane_uv).pVtbl.as_ref().unwrap().GetHeight.unwrap()(plane_uv);
        let line_uv = (*plane_uv).pVtbl.as_ref().unwrap().GetHPitch.unwrap()(plane_uv);

        let data_uv =(*plane_uv).pVtbl.as_ref().unwrap().GetNative.unwrap()(plane_uv) as *mut u8;

        for y in 0..height_uv {
            let data_line = data_uv.add(y as usize * line_uv as usize);
            for x in 0..width_uv {
                std::ptr::write(data_line.add(x as usize * 2), U);
                std::ptr::write(data_line.add(x as usize * 2 + 1), V);
            }
        }
    }
}