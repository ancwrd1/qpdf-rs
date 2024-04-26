# qpdf-rs

[![github actions](https://github.com/ancwrd1/qpdf-rs/workflows/CI/badge.svg)](https://github.com/ancwrd1/qpdf-rs/actions)
[![crates](https://img.shields.io/crates/v/qpdf.svg)](https://crates.io/crates/qpdf)
[![license](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![license](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![docs.rs](https://docs.rs/qpdf/badge.svg)](https://docs.rs/qpdf)

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

The prebuilt bindings for those targets are included in the source tree.

By default, `pkg-config` will be used to link against the system library `libqpdf`.

If the `vendored` feature is enabled, a vendored source tree of qpdf is built and linked statically.

The `legacy` feature enables bindings to the r2/3/4 encryption options which are available in qpdf 10.x but not 11.x.

## Usage example

```rust,no_run
use qpdf::*;

fn make_pdf_from_scratch() -> qpdf::Result<Vec<u8>> {
    let qpdf = QPdf::empty();

    let font = qpdf
        .parse_object(
            r#"<<
                        /Type /Font
                        /Subtype /Type1
                        /Name /F1
                        /BaseFont /Helvetica
                        /Encoding /WinAnsiEncoding
                      >>"#,
        )?;

    let procset = qpdf.parse_object("[/PDF /Text]")?;
    let contents = qpdf.new_stream(b"BT /F1 15 Tf 72 720 Td (First Page) Tj ET\n");
    let mediabox = qpdf.parse_object("[0 0 612 792]")?;
    let rfont = qpdf.new_dictionary_from([("/F1", font.into_indirect())]);
    let resources = qpdf.new_dictionary_from([
        ("/ProcSet", procset.into_indirect()),
        ("/Font", rfont.into())
    ]);
    let page = qpdf.new_dictionary_from([
        ("/Type", qpdf.new_name("/Page")),
        ("/MediaBox", mediabox),
        ("/Contents", contents.into()),
        ("/Resources", resources.into()),
    ]);

    qpdf.add_page(&page.into_indirect(), true)?;

    let mem = qpdf
        .writer()
        .static_id(true)
        .force_pdf_version("1.7")
        .normalize_content(true)
        .preserve_unreferenced_objects(false)
        .object_stream_mode(ObjectStreamMode::Preserve)
        .compress_streams(false)
        .stream_data_mode(StreamDataMode::Preserve)
        .write_to_memory()?;

    Ok(mem)
}
```

## Additional build requirements

* C/C++ compiler
* For the targets which do not have prebuilt bindgen bindings:
  * Installed clang/llvm (with `libclang` shared library) for bindgen build-time invocation
  * For cross-compilation a custom sysroot must be passed to clang via `BINDGEN_EXTRA_CLANG_ARGS`
    environment variable, for example: `BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/x86_64-w64-mingw32/sys-root"`

## License

Licensed under [Apache 2.0](https://opensource.org/licenses/Apache-2.0) license.
