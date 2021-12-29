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

## Usage example

```rust,no_run
fn make_pdf_from_scratch() -> qpdf::Result<Vec<u8>> {
    let qpdf = Qpdf::empty();

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
    let resources = qpdf.new_dictionary_from([("/ProcSet", procset.into_indirect()), ("/Font", rfont.into())]);
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

    Ok(mem.as_ref().to_vec())
}
```

## Additional build requirements

* C/C++ compiler
* Installed clang/llvm (with `libclang` shared library) for bindgen build-time invocation
* For cross-compilation a custom sysroot must be passed to clang via `BINDGEN_EXTRA_CLANG_ARGS`
   environment variable, for example: `BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/x86_64-w64-mingw32/sys-root"`

## License

Licensed under [Apache 2.0](https://opensource.org/licenses/Apache-2.0) license.
