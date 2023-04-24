#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#[allow(dead_code)]
//
// Notice Regarding Standards.  AMD does not provide a license or sublicense to
// any Intellectual Property Rights relating to any standards, including but not
// limited to any audio and/or video codec technologies such as MPEG-2, MPEG-4;
// AVC/H.264; HEVC/H.265; AAC decode/FFMPEG; AAC encode/FFMPEG; VC-1; and MP3
// (collectively, the "Media Technologies"). For clarity, you will pay any
// royalties due for such third party technologies, which may include the Media
// Technologies that are owed as a result of AMD providing the Software to you.
//
// MIT license
//
//
// Copyright (c) 2018 Advanced Micro Devices, Inc. All rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
//

// this sample encodes NV12 frames using AMF Encoder and writes them to H.264 elmentary stream
mod amf_bindings;
use amf_bindings::*;
mod amf_wrappers;

use amf_wrappers::*;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

static memoryTypeIn: AMF_MEMORY_TYPE = AMF_MEMORY_TYPE_AMF_MEMORY_DX11;
static formatIn: AMF_SURFACE_FORMAT = AMF_SURFACE_FORMAT_AMF_SURFACE_NV12;

static widthIn: amf_int32 = 1920;
static heightIn: amf_int32 = 1080;
static frameRateIn: amf_int32 = 30;
static bitRateIn: amf_int64 = 5000000i64;
static rectSize: amf_int32 = 50;
static frameCount: amf_int32 = 500;

static mut xPos: amf_int32 = 0;
static mut yPos: amf_int32 = 0;

static mut pColor1: *mut AMFSurface = std::ptr::null_mut();
static mut pColor2: *mut AMFSurface = std::ptr::null_mut();

fn main() -> std::io::Result<()> {
    amf_factory_helper_init().expect("AMFFactoryHelper_Init failed");
    println!("AMFFactoryHelper_Init succeeded");
    let level = AMF_TRACE_DEBUG as i32;
    let previous_level = amf_trace_set_global_level(level);
    println!("AMFTraceSetGlobalLevel result: {:?}", previous_level);
    let console_str = "Console";
    let mut is_ok = amf_trace_enable_writer(console_str, true);
    println!("AMFTraceEnableWriter result: {:?}", is_ok);
    is_ok = amf_trace_enable_writer("DebugOutput", true);
    println!("AMFTraceEnableWriter result: {:?}", is_ok);
    let previous_level = amf_trace_set_writer_level(console_str, level);
    println!("AMFTraceSetWriterLevel result: {:?}", previous_level);

    let factory = get_amf_factory().expect("Failed to get AMF factory");
    let mut context = create_amf_context(factory).expect("Failed to create AMF context");
    let res_init_dx11 = init_dx11(context);
    println!("InitDX11 result: {:?}", res_init_dx11);
    if res_init_dx11 != AMF_RESULT_AMF_OK {
        panic!("InitDX11 failed");
    }

    PrepareFillDX11(context);

    let (res_create_component, mut encoder) =
        create_component(factory, context, "AMFVideoEncoderVCE_AVC");

    println!("CreateComponent result: {:?}", res_create_component);
    if res_create_component != AMF_RESULT_AMF_OK {
        panic!("CreateComponent failed");
    }
    let mut res = AMF_RESULT_AMF_OK;
    let size: AMFSize = AMFConstructSize(widthIn, heightIn);
    let framerate: AMFRate = AMFConstructRate(frameRateIn.try_into().unwrap(), 1);
    let usage = "Usage";
    AMFAssignPropertyInt64(
        &mut res,
        encoder,
        usage,
        AMF_VIDEO_ENCODER_USAGE_ENUM_AMF_VIDEO_ENCODER_USAGE_TRANSCODING as i64,
    );
    println!("AMFAssignPropertyInt64 usage result: {:?}", res);
    let bitrate = "TargetBitrate";
    AMFAssignPropertyInt64(&mut res, encoder, bitrate, bitRateIn);
    println!("AMFAssignPropertyInt64 bitrate result: {:?}", res);
    let frameSize = "FrameSize";
    AMFAssignPropertySize(&mut res, encoder, frameSize, size);
    println!("AMFAssignPropertySize frameSize result: {:?}", res);

    let res_init_encoder = init_encoder(encoder, formatIn, widthIn, heightIn);
    println!("Encoder Init() result: {:?}", res_init_encoder);
    if res_init_encoder != AMF_RESULT_AMF_OK {
        panic!("Encoder Init() failed");
    }
    let mut submitted = 0;
    let mut file = File::create(Path::new("./output.mp4"))?;
    let mut surfaceIn: *mut AMFSurface = std::ptr::null_mut();

    while submitted < frameCount {
        if surfaceIn.is_null() {
            let result = alloc_surface(context, memoryTypeIn, formatIn, widthIn, heightIn);
            match result {
                Ok(surface) => {
                    println!("AllocSurface() result: {:?}", AMF_RESULT_AMF_OK);
                    surfaceIn = surface;
                    FillSurfaceDX11(context, surfaceIn)?;
                }
                Err(err) => {
                    println!("AllocSurface() failed with error: {:?}", err);
                    break;
                }
            }
        }
        let submit_result = submit_input(encoder, surfaceIn);
        if submit_result.is_ok() {
            release(surfaceIn);
            surfaceIn = std::ptr::null_mut();
            assert_eq!(submit_result.unwrap(), (), "SubmitInput() failed");
            submitted += 1;
        }
        let query_result = query_output(encoder);
        match query_result {
            Ok(data) => {
                if !data.is_null() {
                    let guid = IID_AMFBuffer();
                    match query_interface(data, &guid) {
                        Ok(buffer) => {
                            write_amf_buffer_to_file(&mut file, buffer)?;
                            release_buffer(buffer);
                            release_data(data);
                        }
                        Err(err) => {
                            println!("QueryInterface() failed with error: {:?}", err);
                            break;
                        }
                    }
                }
            }
            Err(AMF_RESULT_AMF_EOF) => {
                break; // Drain complete
            }
            Err(AMF_RESULT_AMF_REPEAT) => {
                println!("Waiting SubmitInput(), will repeat after sleep");
                thread::sleep(Duration::from_millis(5));
            }
            Err(err) => {
                println!("QueryOutput() failed with error: {:?}", err);
                break;
            }
        }
    }
    file.flush()?;
    drop(file); // Close the output file

    if !surfaceIn.is_null() {
        release_surface(surfaceIn);
    }
    terminate_encoder(encoder);
    release_encoder(encoder);
    encoder = std::ptr::null_mut();
    terminate_context(context);
    release_context(context);
    context = std::ptr::null_mut();
    amf_factory_helper_terminate();
    Ok(())
}

fn AMFConstructSize(width: i32, height: i32) -> AMFSize {
    AMFSize { width, height }
}

fn AMFConstructRate(num: u32, den: u32) -> AMFRate {
    AMFRate { num, den }
}

fn PrepareFillDX11(
    context: *mut AMFContext
) {
    let result = alloc_surface(
        context,
        AMF_MEMORY_TYPE_AMF_MEMORY_HOST,
        formatIn,
        widthIn,
        heightIn,
    );
    match result {
        Ok(surface) => {
            println!("AllocSurface() result for pColor1: {:?}", AMF_RESULT_AMF_OK);
            unsafe {pColor1 = surface;}
        }
        Err(err) => {
            println!("AllocSurface() for pColor1 failed with error: {:?}", err);
        }
    }

    let result = alloc_surface(
        context,
        AMF_MEMORY_TYPE_AMF_MEMORY_HOST,
        formatIn,
        rectSize,
        rectSize,
    );
    match result {
        Ok(surface) => {
            println!("AllocSurface() result for pColor2: {:?}", AMF_RESULT_AMF_OK);
            unsafe { pColor2 = surface; }
        }
        Err(err) => {
            println!("AllocSurface() for pColor2 failed with error: {:?}", err);
        }
    }
    unsafe {
        FillNV12SurfaceWithColor(pColor2, 128, 0, 128);
        FillNV12SurfaceWithColor(pColor1, 128, 255, 128);
        convert_surface(pColor1, memoryTypeIn);
        convert_surface(pColor2, memoryTypeIn);
    }
}

fn FillNV12SurfaceWithColor(surface: *mut AMFSurface, Y: u8, U: u8, V: u8) {
    let plane_y = get_plane_at(surface, 0).expect("Failed to get plane Y");
    let plane_uv = get_plane_at(surface, 1).expect("Failed to get plane UV");
    let width_y = get_width(plane_y);
    let height_y = get_height(plane_y);
    let line_y = get_h_pitch(plane_y);
    let data_y = get_native_plane(plane_y);
    for y in 0..height_y {
        unsafe {
            let data_line = data_y.add(y as usize * line_y as usize);
            std::ptr::write_bytes(data_line, Y, width_y as usize);
        }
    }
    let width_uv = get_width(plane_uv);
    let height_uv = get_height(plane_uv);
    let line_uv = get_h_pitch(plane_uv);
    let data_uv = get_native_plane(plane_uv);
    for y in 0..height_uv {
        unsafe {
            let data_line = data_uv.add(y as usize * line_uv as usize);
            for x in 0..width_uv {
                std::ptr::write(data_line.add(x as usize * 2), U);
                std::ptr::write(data_line.add(x as usize * 2 + 1), V);
            }
        }
    }
}

fn FillSurfaceDX11(context: *mut AMFContext, surface: *mut AMFSurface) -> Result<(), std::io::Error> {
    let mut device_context_dx11: *mut ID3D11DeviceContext = std::ptr::null_mut();

    unsafe {
        // Get native DX objects
        let device_dx11: *mut ID3D11Device =
            (*context).pVtbl.as_ref().unwrap().GetDX11Device.unwrap()(
                context,
                AMF_DX_VERSION_AMF_DX11_0,
            ) as *mut ID3D11Device;
        let plane: *mut AMFPlane =
            (*surface).pVtbl.as_ref().unwrap().GetPlaneAt.unwrap()(surface, 0);
        let surface_dx11: *mut ID3D11Texture2D =
            (*plane).pVtbl.as_ref().unwrap().GetNative.unwrap()(plane) as *mut ID3D11Texture2D;
        (*device_dx11)
            .lpVtbl
            .as_ref()
            .unwrap()
            .GetImmediateContext
            .unwrap()(device_dx11, &mut device_context_dx11);

        // Fill the surface with color1
        let planeColor1: *mut AMFPlane =
            (*pColor1).pVtbl.as_ref().unwrap().GetPlaneAt.unwrap()(pColor1, 0);
        let surface_dx11_color1: *mut ID3D11Texture2D =
            (*planeColor1).pVtbl.as_ref().unwrap().GetNative.unwrap()(planeColor1)
                as *mut ID3D11Texture2D;
        (*device_context_dx11)
            .lpVtbl
            .as_ref()
            .unwrap()
            .CopyResource
            .unwrap()(
            device_context_dx11,
            surface_dx11 as *mut ID3D11Resource,
            surface_dx11_color1 as *mut ID3D11Resource,
        );

        if xPos + rectSize > widthIn {
            xPos = 0;
        }
        if yPos + rectSize > heightIn {
            yPos = 0;
        }

        // Fill the surface with color2
        let rect = D3D11_BOX {
            left: 0,
            top: 0,
            front: 0,
            right: rectSize as u32,
            bottom: rectSize as u32,
            back: 1,
        };

        let planeColor2: *mut AMFPlane =
            (*pColor2).pVtbl.as_ref().unwrap().GetPlaneAt.unwrap()(pColor2, 0);
        let surface_dx11_color2: *mut ID3D11Texture2D =
            (*planeColor2).pVtbl.as_ref().unwrap().GetNative.unwrap()(planeColor2)
                as *mut ID3D11Texture2D;
        (*device_context_dx11)
            .lpVtbl
            .as_ref()
            .unwrap()
            .CopySubresourceRegion
            .unwrap()(
            device_context_dx11,
            surface_dx11 as *mut ID3D11Resource,
            0,
            xPos as u32,
            yPos as u32,
            0,
            surface_dx11_color2 as *mut ID3D11Resource,
            0,
            &rect,
        );

        (*device_context_dx11)
            .lpVtbl
            .as_ref()
            .unwrap()
            .Flush
            .unwrap()(device_context_dx11);

        // Update x_pos and y_pos
        xPos += 2;
        yPos += 2;

        // Release device context
        (*device_context_dx11)
            .lpVtbl
            .as_ref()
            .unwrap()
            .Release
            .unwrap()(device_context_dx11);
    }

    Ok(())
}
