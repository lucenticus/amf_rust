# AMF Simple Encoder in Rust language

This repo contains example of using AMD Advanced Media Framework [AMF](https://github.com/GPUOpen-LibrariesAndSDKs/AMF) with Rust language.
Implementation is based on the [SimpleEncoderC](https://github.com/GPUOpen-LibrariesAndSDKs/AMF/tree/master/amf/public/samples/SamplesC/SimpleEncoderC) example from AMF repository.
Please note that this is just experimental implementation and might contains some issues.

## Prerequisites
* Windows 10+
* AMD GPU
* Git
* Rust toolchain

## Usage

1. Clone the repo with AMF submodule:
```bash
git clone --recursive https://github.com/lucenticus/amf_rust.git
cd amf_rust
 ```

2. Build & run example:
```bash
cargo build
cargo run
 ```

3. The result is located in the file `output.mp4`
