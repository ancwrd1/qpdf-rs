# Rust bindings to QPDF C++ library

## Overview

This project contains Rust safe bindings to a popular [QPDF C++ library](https://github.com/qpdf/qpdf).
It uses the QPDF C API exposed via `qpdf-c.h` header.

Tested on the following targets:

* x86_64-unknown-linux-gnu
* aarch64-unknown-linux-gnu
* x86_64-pc-windows-gnu
* x86_64-pc-windows-msvc
* x86_64-apple-darwin
* aarch64-apple-darwin

## Additional build requirements

* C/C++ compiler
* Installed clang/llvm (with `libclang` shared library) for bindgen build-time invocation
* For cross-compilation a custom sysroot must be passed to clang via `BINDGEN_EXTRA_CLANG_ARGS`
   environment variable, for example: `BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/x86_64-w64-mingw32/sys-root"`

## License

Licensed under [Apache 2.0](https://opensource.org/licenses/Apache-2.0) license.
