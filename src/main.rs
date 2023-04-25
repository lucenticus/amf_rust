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

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#[allow(dead_code)]
mod amf_bindings;
use amf_bindings::*;
use winapi::um::d3d11::{ID3D11Resource, ID3D11Texture2D};
mod amf_wrappers;

use amf_wrappers::*;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

const memoryTypeIn: AMF_MEMORY_TYPE = AMF_MEMORY_TYPE_AMF_MEMORY_DX11;
const formatIn: AMF_SURFACE_FORMAT = AMF_SURFACE_FORMAT_AMF_SURFACE_NV12;

const widthIn: amf_int32 = 1920;
const heightIn: amf_int32 = 1080;
const bitRateIn: amf_int64 = 5000000i64;
const rectSize: amf_int32 = 50;
const frameCount: amf_int32 = 500;

fn main() -> std::io::Result<()> {
    amf_factory_helper_init().expect("AMFFactoryHelper_Init failed");
    println!("AMFFactoryHelper_Init succeeded");

    init_debug_output();

    let factory = get_amf_factory().expect("Failed to get AMF factory");
    let context = create_amf_context(factory).expect("Failed to create AMF context");
    let mut res = init_dx11(context);
    if res != AMF_RESULT_AMF_OK {
        panic!("InitDX11 failed with result: {:?}", res);
    }
    let mut pColor1: *mut AMFSurface = std::ptr::null_mut();
    let mut pColor2: *mut AMFSurface = std::ptr::null_mut();
    PrepareFillDX11(context, &mut pColor1, &mut pColor2);

    let (res_encoder, encoder) =
        create_component(factory, context, "AMFVideoEncoderVCE_AVC");

    if res_encoder != AMF_RESULT_AMF_OK {
        panic!("CreateComponent failed with result: {:?}", res_encoder);
    }

    // Setting parameters for encoder
    AMFAssignPropertyInt64(
        &mut res,
        encoder,
        "Usage",
        AMF_VIDEO_ENCODER_USAGE_ENUM_AMF_VIDEO_ENCODER_USAGE_TRANSCODING as i64,
    );
    println!("AMFAssignPropertyInt64 usage result: {:?}", res);

    AMFAssignPropertyInt64(&mut res, encoder, "TargetBitrate", bitRateIn);
    println!("AMFAssignPropertyInt64 bitrate result: {:?}", res);

    let size: AMFSize = AMFSize { width: widthIn, height: heightIn };
    AMFAssignPropertySize(&mut res, encoder, "FrameSize", size);
    println!("AMFAssignPropertySize frameSize result: {:?}", res);

    res = init_encoder(encoder, formatIn, widthIn, heightIn);

    if res != AMF_RESULT_AMF_OK {
        panic!("Encoder Init() failed with result: {:?}", res);
    }
    let mut submitted = 0;
    let mut file = File::create(Path::new("./output.mp4"))?;
    let mut surfaceIn: *mut AMFSurface = std::ptr::null_mut();

    let mut xPos: amf_int32 = 0;
    let mut yPos: amf_int32 = 0;

    while submitted < frameCount {
        if surfaceIn.is_null() {
            let result = alloc_surface(context, memoryTypeIn, formatIn, widthIn, heightIn);
            match result {
                Ok(surface) => {
                    surfaceIn = surface;
                    FillSurfaceDX11(
                        context,
                        surfaceIn,
                        &mut pColor1,
                        &mut pColor2,
                        &mut xPos,
                        &mut yPos,
                    )?;
                }
                Err(err) => {
                    println!("AllocSurface() failed with error: {:?}", err);
                    break;
                }
            }
        }
        // Submit the input surface into encoder
        let submit_result = submit_input(encoder, surfaceIn);
        if submit_result.is_ok() {
            release(surfaceIn);
            surfaceIn = std::ptr::null_mut();
            assert_eq!(submit_result.unwrap(), (), "SubmitInput() failed");
            submitted += 1;
        }
        // Receive output from encoder
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
    terminate_context(context);
    release_context(context);
    amf_factory_helper_terminate();
    Ok(())
}

fn init_debug_output() {
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
}

fn PrepareFillDX11(
    context: *mut AMFContext,
    pColor1: &mut *mut AMFSurface,
    pColor2: &mut *mut AMFSurface,
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
            *pColor1 = surface;
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
            *pColor2 = surface;
        }
        Err(err) => {
            println!("AllocSurface() for pColor2 failed with error: {:?}", err);
        }
    }
    FillNV12SurfaceWithColor(*pColor2, 128, 0, 128);
    FillNV12SurfaceWithColor(*pColor1, 128, 255, 128);
    convert_surface(*pColor1, memoryTypeIn);
    convert_surface(*pColor2, memoryTypeIn);
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

fn FillSurfaceDX11(
    context: *mut AMFContext,
    surface: *mut AMFSurface,
    p_color1: &mut *mut AMFSurface,
    p_color2: &mut *mut AMFSurface,
    xPos: &mut amf_int32,
    yPos: &mut amf_int32,
) -> Result<(), std::io::Error> {
    let device_dx11 = get_dx11_device(context);

    let plane = get_plane_at(surface, 0).expect("Failed to get plane");
    let surface_dx11 = get_native_plane(plane);

    let device_context_dx11 = get_immediate_context(device_dx11);

    // Fill the surface with color1
    let plane_color1 = get_plane_at(*p_color1, 0).expect("Failed to get plane plane_color1");
    let surface_dx11_color1 = get_native_plane(plane_color1) as *mut ID3D11Texture2D;

    copy_resource(
        device_context_dx11,
        surface_dx11_color1.cast::<ID3D11Resource>(),
        surface_dx11.cast::<ID3D11Resource>(),
    );

    if *xPos + rectSize > widthIn {
        *xPos = 0;
    }
    if *yPos + rectSize > heightIn {
        *yPos = 0;
    }

    // Fill the surface with color2
    let rect = winapi::um::d3d11::D3D11_BOX {
        left: 0,
        top: 0,
        front: 0,
        right: rectSize as u32,
        bottom: rectSize as u32,
        back: 1,
    };

    let plane_color2 = get_plane_at(*p_color2, 0).expect("Failed to get plane plane_color2");
    let surface_dx11_color2 = get_native_plane(plane_color2) as *mut ID3D11Texture2D;

    copy_subresource_region(
        device_context_dx11,
        surface_dx11.cast::<ID3D11Resource>(),
        0,
        *xPos as u32,
        *yPos as u32,
        0,
        surface_dx11_color2.cast::<ID3D11Resource>(),
        0,
        &rect,
    );

    flush_device_context(device_context_dx11);

    // Update xPos and yPos
    *xPos += 2;
    *yPos += 2;

    // Release device context
    release_device_context_dx11(device_context_dx11);

    Ok(())
}
