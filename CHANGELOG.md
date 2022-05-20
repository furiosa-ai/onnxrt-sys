# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2022-05-20

### Added

- Support ONNX Runtime v1.11.0.

## [0.8.0] - 2022-05-17

### Added

- Support ONNX Runtime v1.10.0.

## [0.7.0] - 2022-03-11

### Added

- Support Apple M1 again.

## [0.6.0] - 2022-01-13

### Added

- Can specify the separate directories of ONNX Runtime header files and library
  by setting the `ORT_INCLUDE_DIR` and `ORT_LIB_DIR` environment variables when
  `ORT_STRATEGY=system`.

### Removed

- The `ORT_LIB_LOCATION` environment variable, which is superseded by
  `ORT_INCLUDE_DIR` and `ORT_LIB_DIR`.

## [0.5.0] - 2022-01-04

### Added

- Support ONNX Runtime v1.9.0.

## [0.4.0] - 2021-09-24

### Added

- Support Apple M1.

## [0.3.0] - 2021-08-23

### Added

- Support ONNX Runtime v1.8.1.

## [0.2.0] - 2021-08-20

### Added

- Support ONNX Runtime v1.8.0.

## [0.1.0] - 2021-07-27

### Added

- Support ONNX Runtime v1.7.

[Unreleased]: https://github.com/furiosa-ai/onnxrt-sys/compare/0.9.0...HEAD
[0.9.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.9.0
[0.8.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.8.0
[0.7.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.7.0
[0.6.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.6.0
[0.5.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.5.0
[0.4.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.4.0
[0.3.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.3.0
[0.2.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.2.0
[0.1.0]: https://github.com/furiosa-ai/onnxrt-sys/releases/tag/0.1.0
