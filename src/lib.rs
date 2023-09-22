#![doc(html_root_url = "https://furiosa-ai.github.io/onnxrt-sys/0.18.0/onnxrt_sys")]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Disable clippy and `u128` not being FFI-safe (see #1)
#![allow(clippy::all)]
#![allow(improper_ctypes)]
#![warn(rust_2018_idioms)]

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/generated/bindings.rs"));

#[cfg(target_os = "windows")]
pub type OnnxEnumInt = i32;
#[cfg(not(target_os = "windows"))]
pub type OnnxEnumInt = u32;
