# OnnxRt-sys

[![build status](https://github.com/furiosa-ai/onnxrt-sys/actions/workflows/build.yml/badge.svg)](https://github.com/furiosa-ai/onnxrt-sys/actions/workflows/build.yml?query=branch%3Amain)
[![docs](https://img.shields.io/badge/docs-0.21.1-blue.svg)](https://furiosa-ai.github.io/onnxrt-sys/0.21.1/onnxrt_sys/)

OnnxRt-sys provides low-level Rust bindings to [the C API] for Microsoft's
[ONNX Runtime] v1.16.3.

[the C API]: https://www.onnxruntime.ai/docs/reference/api/c-api.html
[ONNX Runtime]: https://www.onnxruntime.ai/

## Requirements

### Rust

This program targets the latest stable version of Rust 1.56.1 or later.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
onnxrt-sys = { git = "https://github.com/furiosa-ai/onnxrt-sys", tag = "0.21.1" }
```

## Example

See the `examples` directory for examples on how to use the library.

## License and Origin

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

This started as a fork of Nicolas Bigaouette's [onnxruntime-sys].

[onnxruntime-sys]: https://github.com/nbigaouette/onnxruntime-rs/tree/v0.0.12/onnxruntime-sys

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
