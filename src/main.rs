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

const MEMORY_TYPE_IN: AMF_MEMORY_TYPE = AMF_MEMORY_TYPE_AMF_MEMORY_DX11;
const FORMAT_IN: AMF_SURFACE_FORMAT = AMF_SURFACE_FORMAT_AMF_SURFACE_NV12;

const WIDTH_IN: amf_int32 = 1920;
const HEIGHT_IN: amf_int32 = 1080;
const BITRATE_IN: amf_int64 = 5000000i64;
const RECT_SIZE: amf_int32 = 50;
const FRAME_COUNT: amf_int32 = 500;

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
    let mut p_color1: *mut AMFSurface = std::ptr::null_mut();
    let mut p_color2: *mut AMFSurface = std::ptr::null_mut();
    prepare_fill_dx11(context, &mut p_color1, &mut p_color2);

    let (res_encoder, encoder) =
        create_component(factory, context, "AMFVideoEncoderVCE_AVC");

    if res_encoder != AMF_RESULT_AMF_OK {
        panic!("CreateComponent failed with result: {:?}", res_encoder);
    }

    // Setting parameters for encoder
    amf_assign_property_int64(
        &mut res,
        encoder,
        "Usage",
        AMF_VIDEO_ENCODER_USAGE_ENUM_AMF_VIDEO_ENCODER_USAGE_TRANSCODING as i64,
    );
    println!("AMFAssignPropertyInt64 usage result: {:?}", res);

    amf_assign_property_int64(&mut res, encoder, "TargetBitrate", BITRATE_IN);
    println!("AMFAssignPropertyInt64 bitrate result: {:?}", res);

    let size: AMFSize = AMFSize { width: WIDTH_IN, height: HEIGHT_IN };
    amf_assign_property_size(&mut res, encoder, "FrameSize", size);
    println!("AMFAssignPropertySize frame size result: {:?}", res);

    res = init_encoder(encoder, FORMAT_IN, WIDTH_IN, HEIGHT_IN);

    if res != AMF_RESULT_AMF_OK {
        panic!("init_encoder() failed with result: {:?}", res);
    }
    let mut submitted = 0;
    let mut file = File::create(Path::new("./output.mp4"))?;
    let mut surface_in: *mut AMFSurface = std::ptr::null_mut();

    let mut x_pos: amf_int32 = 0;
    let mut y_pos: amf_int32 = 0;

    while submitted < FRAME_COUNT {
        if surface_in.is_null() {
            let result = alloc_surface(context, MEMORY_TYPE_IN, FORMAT_IN, WIDTH_IN, HEIGHT_IN);
            match result {
                Ok(surface) => {
                    surface_in = surface;
                    fill_surface_dx11(
                        context,
                        surface_in,
                        &mut p_color1,
                        &mut p_color2,
                        &mut x_pos,
                        &mut y_pos,
                    )?;
                }
                Err(err) => {
                    println!("alloc_surface() failed with error: {:?}", err);
                    break;
                }
            }
        }
        // Submit the input surface into encoder
        let submit_result = submit_input(encoder, surface_in);
        if submit_result.is_ok() {
            release(surface_in);
            surface_in = std::ptr::null_mut();
            assert_eq!(submit_result.unwrap(), (), "submit_input() failed");
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
                            println!("query_interface() failed with error: {:?}", err);
                            break;
                        }
                    }
                }
            }
            Err(AMF_RESULT_AMF_EOF) => {
                break; // Drain complete
            }
            Err(AMF_RESULT_AMF_REPEAT) => {
                println!("Waiting submit_input(), will repeat after sleep");
                thread::sleep(Duration::from_millis(5));
            }
            Err(err) => {
                println!("query_output() failed with error: {:?}", err);
                break;
            }
        }
    }
    file.flush()?;
    drop(file); // Close the output file

    if !surface_in.is_null() {
        release_surface(surface_in);
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
    println!("amf_trace_set_global_level result: {:?}", previous_level);
    let console_str = "Console";
    let mut is_ok = amf_trace_enable_writer(console_str, true);
    println!("amf_trace_enable_writer result: {:?}", is_ok);
    is_ok = amf_trace_enable_writer("DebugOutput", true);
    println!("amf_trace_enable_writer result: {:?}", is_ok);
    let previous_level = amf_trace_set_writer_level(console_str, level);
    println!("amf_trace_set_writer_level result: {:?}", previous_level);
}

fn prepare_fill_dx11(
    context: *mut AMFContext,
    p_color1: &mut *mut AMFSurface,
    p_color2: &mut *mut AMFSurface,
) {
    let result = alloc_surface(
        context,
        AMF_MEMORY_TYPE_AMF_MEMORY_HOST,
        FORMAT_IN,
        WIDTH_IN,
        HEIGHT_IN,
    );
    match result {
        Ok(surface) => {
            println!("alloc_surface() result for pColor1: {:?}", AMF_RESULT_AMF_OK);
            *p_color1 = surface;
        }
        Err(err) => {
            println!("alloc_surface() for p_color1 failed with error: {:?}", err);
        }
    }

    let result = alloc_surface(
        context,
        AMF_MEMORY_TYPE_AMF_MEMORY_HOST,
        FORMAT_IN,
        RECT_SIZE,
        RECT_SIZE,
    );
    match result {
        Ok(surface) => {
            println!("alloc_surface() result for p_color2: {:?}", AMF_RESULT_AMF_OK);
            *p_color2 = surface;
        }
        Err(err) => {
            println!("alloc_surface() for p_color2 failed with error: {:?}", err);
        }
    }
    fill_nv12_surface_with_color(*p_color2, 128, 0, 128);
    fill_nv12_surface_with_color(*p_color1, 128, 255, 128);
    convert_surface(*p_color1, MEMORY_TYPE_IN);
    convert_surface(*p_color2, MEMORY_TYPE_IN);
}

fn fill_nv12_surface_with_color(surface: *mut AMFSurface, Y: u8, U: u8, V: u8) {
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

fn fill_surface_dx11(
    context: *mut AMFContext,
    surface: *mut AMFSurface,
    p_color1: &mut *mut AMFSurface,
    p_color2: &mut *mut AMFSurface,
    x_pos: &mut amf_int32,
    y_pos: &mut amf_int32,
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

    if *x_pos + RECT_SIZE > WIDTH_IN {
        *x_pos = 0;
    }
    if *y_pos + RECT_SIZE > HEIGHT_IN {
        *y_pos = 0;
    }

    // Fill the surface with color2
    let rect = winapi::um::d3d11::D3D11_BOX {
        left: 0,
        top: 0,
        front: 0,
        right: RECT_SIZE as u32,
        bottom: RECT_SIZE as u32,
        back: 1,
    };

    let plane_color2 = get_plane_at(*p_color2, 0).expect("Failed to get plane plane_color2");
    let surface_dx11_color2 = get_native_plane(plane_color2) as *mut ID3D11Texture2D;

    copy_subresource_region(
        device_context_dx11,
        surface_dx11.cast::<ID3D11Resource>(),
        0,
        *x_pos as u32,
        *y_pos as u32,
        0,
        surface_dx11_color2.cast::<ID3D11Resource>(),
        0,
        &rect,
    );

    flush_device_context(device_context_dx11);

    // Update xPos and yPos
    *x_pos += 2;
    *y_pos += 2;

    // Release device context
    release_device_context_dx11(device_context_dx11);

    Ok(())
}
