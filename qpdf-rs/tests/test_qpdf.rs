use std::collections::HashSet;

use qpdf::scalar::QPdfScalar;
use qpdf::*;

fn load_pdf() -> QPdf {
    QPdf::read("tests/data/test.pdf").unwrap()
}

#[test]
fn test_qpdf_version() {
    assert_eq!(QPdf::library_version(), "10.6.3");
    println!("{}", QPdf::library_version());
}

#[test]
fn test_writer() {
    let qpdf = load_pdf();
    let mut writer = qpdf.writer();
    writer
        .force_pdf_version("1.7")
        .normalize_content(true)
        .preserve_unreferenced_objects(true)
        .object_stream_mode(ObjectStreamMode::Disable)
        .linearize(true)
        .compress_streams(true)
        .stream_data_mode(StreamDataMode::Compress);

    let mem = writer.write_to_memory().unwrap();

    let mem_pdf = QPdf::read_from_memory(&mem).unwrap();
    assert_eq!(mem_pdf.get_pdf_version(), "1.7");
    assert!(mem_pdf.is_linearized());
}

#[test]
fn test_pdf_from_scratch() {
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
        )
        .unwrap();

    let procset = qpdf.parse_object("[/PDF /Text]").unwrap();
    let contents = qpdf.new_stream(b"BT /F1 15 Tf 72 720 Td (First Page) Tj ET\n");
    let mediabox = qpdf.parse_object("[0 0 612 792]").unwrap();
    let rfont = qpdf.new_dictionary_from([("/F1", font.into_indirect())]);
    let resources = qpdf.new_dictionary_from([("/ProcSet", procset.into_indirect()), ("/Font", rfont.into())]);
    let page = qpdf.new_dictionary_from([
        ("/Type", qpdf.new_name("/Page")),
        ("/MediaBox", mediabox),
        ("/Contents", contents.into()),
        ("/Resources", resources.into()),
    ]);

    qpdf.add_page(&page.into_indirect(), true).unwrap();

    let mem = qpdf
        .writer()
        .static_id(true)
        .force_pdf_version("1.7")
        .normalize_content(true)
        .preserve_unreferenced_objects(true)
        .object_stream_mode(ObjectStreamMode::Preserve)
        .linearize(true)
        .compress_streams(true)
        .stream_data_mode(StreamDataMode::Preserve)
        .write_to_memory()
        .unwrap();

    let mem_pdf = QPdf::read_from_memory(&mem).unwrap();
    assert_eq!(mem_pdf.get_pdf_version(), "1.7");
    assert!(mem_pdf.is_linearized());
}

#[test]
fn test_qpdf_basic_objects() {
    let qpdf = QPdf::empty();
    let obj = qpdf.new_bool(true);
    assert!(obj.get_type() == QPdfObjectType::Boolean && obj.as_bool());
    assert_eq!(obj.to_string(), "true");

    let obj = qpdf.new_name("foo");
    assert!(obj.get_type() == QPdfObjectType::Name && obj.as_name() == "foo");
    assert_eq!(obj.to_string(), "foo");

    let obj = qpdf.new_integer(12_3456_7890);
    assert!(obj.is_scalar() && obj.as_i64() == 12_3456_7890);
    assert_eq!(obj.to_string(), "1234567890");

    let obj = qpdf.new_null();
    assert_eq!(obj.get_type(), QPdfObjectType::Null);
    assert_eq!(obj.to_string(), "null");

    let obj = qpdf.new_real(1.2345, 3);
    assert_eq!(obj.as_real(), "1.234");
    assert_eq!(obj.to_string(), "1.234");

    let obj = qpdf.new_stream(&[]);
    assert_eq!(obj.get_type(), QPdfObjectType::Stream);
    assert_eq!(obj.to_string(), "3 0 R");

    obj.get_dictionary().set("/Type", &qpdf.new_name("/Stream"));

    let obj_id = obj.get_id();
    assert_ne!(obj.into_indirect().get_id(), obj_id);
}

#[test]
fn test_qpdf_streams() {
    let qpdf = QPdf::empty();

    let obj = qpdf.get_object_by_id(1234, 1);
    assert!(obj.is_none());

    let obj = qpdf.new_stream_with_dictionary([("/Type", qpdf.new_name("/Test"))], &[1, 2, 3, 4]);
    assert_eq!(obj.get_type(), QPdfObjectType::Stream);

    let by_id: QPdfStream = qpdf
        .get_object_by_id(obj.get_id(), obj.get_generation())
        .unwrap()
        .into();
    println!("{}", by_id);

    let data = by_id.get_data(StreamDecodeLevel::None).unwrap();
    assert_eq!(data.as_ref(), &[1, 2, 3, 4]);

    assert_eq!(obj.get_dictionary().get("/Type").unwrap().as_name(), "/Test");

    let indirect = obj.into_indirect();
    assert!(indirect.is_indirect());
    assert_ne!(indirect.get_id(), 0);
    assert_eq!(indirect.get_generation(), 0);
}

#[test]
fn test_parse_object() {
    let text = "<< /Type /Page /Resources << /XObject null >> /MediaBox null /Contents null >>";
    let qpdf = QPdf::empty();
    let obj = qpdf.parse_object(text).unwrap();
    assert_eq!(obj.get_type(), QPdfObjectType::Dictionary);
    println!("{}", obj);
    println!("version: {}", qpdf.get_pdf_version());
}

#[test]
fn test_error() {
    let qpdf = QPdf::empty();
    assert!(qpdf.get_page(0).is_none());
    let result = qpdf.parse_object("<<--< /Type -- null >>");
    assert!(result.is_err());
    println!("{:?}", result);
}

#[test]
fn test_array() {
    let qpdf = QPdf::empty();
    let mut arr = qpdf.new_array();
    arr.push(&qpdf.new_integer(1));
    arr.push(&qpdf.new_integer(2));
    arr.push(&qpdf.new_integer(3));
    assert_eq!(arr.to_string(), "[ 1 2 3 ]");

    assert!(arr.get(10).is_none());

    assert_eq!(
        arr.iter().map(|v| QPdfScalar::from(v).as_i32()).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    arr.set(1, &qpdf.new_integer(5));
    assert_eq!(arr.to_string(), "[ 1 5 3 ]");
}

#[test]
fn test_dictionary() {
    let qpdf = QPdf::empty();
    let dict: QPdfDictionary = qpdf
        .parse_object("<< /Type /Page /Resources << /XObject null >> /MediaBox [1 2 3 4] /Contents (hello) >>")
        .unwrap()
        .into();

    let keys = dict.keys().into_iter().collect::<HashSet<_>>();
    assert_eq!(
        keys,
        ["/Type", "/Resources", "/MediaBox", "/Contents"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect::<HashSet<_>>()
    );

    assert_eq!(dict.get("/Type").unwrap().get_type(), QPdfObjectType::Name);
    assert_eq!(dict.get("/Contents").unwrap().as_string(), "hello");

    let bval = qpdf.new_bool(true);
    dict.set("/MyKey", &bval);

    let setval = dict.get("/MyKey").unwrap();
    assert!(setval.as_bool());
    assert_ne!(bval, setval);

    dict.remove("/MyKey");
    assert!(dict.get("/MyKey").is_none());
}

#[test]
fn test_strings() {
    let qpdf = QPdf::empty();
    let bin_str = qpdf.new_binary_string(&[1, 2, 3, 4]);
    assert_eq!(bin_str.to_string(), "<01020304>");

    let utf8_str = qpdf.new_utf8_string("привет");
    assert_eq!(utf8_str.to_string(), "<feff043f04400438043204350442>");

    let plain_str = qpdf.new_string("hello");
    assert_eq!(plain_str.to_string(), "(hello)");
    assert_eq!(plain_str.to_binary(), "<68656c6c6f>");
}

#[test]
fn test_pdf_ops() {
    let qpdf = load_pdf();
    println!("{:?}", qpdf.get_pdf_version());

    let trailer = qpdf.get_trailer().unwrap();
    println!("trailer: {}", trailer);

    let root = qpdf.get_root().unwrap();
    println!("root: {}", root);
    assert_eq!(root.get("/Type").unwrap().as_name(), "/Catalog");
    assert!(root.has("/Pages"));

    let pages = qpdf.get_pages().unwrap();
    assert_eq!(pages.len(), 2);

    for page in pages {
        let keys = page.keys();
        assert!(!keys.is_empty());
        println!("{:?}", keys);

        let data = page.get_page_content_data().unwrap();
        println!("{}", String::from_utf8_lossy(data.as_ref()));

        qpdf.add_page(&page, false).unwrap();
    }

    let buffer = qpdf.writer().write_to_memory().unwrap();
    let saved_pdf = QPdf::read_from_memory(&buffer).unwrap();
    assert_eq!(saved_pdf.get_num_pages().unwrap(), 4);

    let pages = saved_pdf.get_pages().unwrap();
    for page in pages {
        saved_pdf.remove_page(&page).unwrap();
    }
    assert_eq!(saved_pdf.get_num_pages().unwrap(), 0);
}

#[test]
fn test_pdf_encrypted() {
    let qpdf = QPdf::read("tests/data/encrypted.pdf");
    assert!(qpdf.is_err());
    println!("{:?}", qpdf);

    let qpdf = QPdf::read_encrypted("tests/data/encrypted.pdf", "test");
    assert!(qpdf.is_ok());

    let data = std::fs::read("tests/data/encrypted.pdf").unwrap();
    let qpdf = QPdf::read_from_memory_encrypted(&data, "test");
    assert!(qpdf.is_ok());
}
