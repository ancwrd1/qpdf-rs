use std::collections::HashSet;

use qpdf::*;

fn load_pdf() -> Qpdf {
    Qpdf::load("tests/data/test.pdf").unwrap()
}

#[test]
fn test_qpdf_version() {
    assert_eq!(Qpdf::library_version(), "10.5.0");
    println!("{}", Qpdf::library_version());
}

#[test]
fn test_qpdf_new_objects() {
    let qpdf = Qpdf::new();
    let obj = qpdf.new_bool(true);
    assert!(obj.is_bool() && obj.as_bool());
    assert_eq!(obj.to_string(), "true");

    let obj = qpdf.new_name("foo");
    assert!(obj.is_name() && obj.as_name() == "foo");
    assert_eq!(obj.to_string(), "foo");

    let obj = qpdf.new_integer(12_3456_7890);
    assert!(obj.is_scalar() && obj.as_i64() == 12_3456_7890);
    assert_eq!(obj.to_string(), "1234567890");

    let obj = qpdf.new_null();
    assert!(obj.is_null());
    assert_eq!(obj.to_string(), "null");

    let obj = qpdf.new_real(1.2345, 3);
    assert_eq!(obj.as_real(), "1.234");
    assert_eq!(obj.to_string(), "1.234");

    let obj = qpdf.new_uninitialized();
    assert!(!obj.is_initialized());

    let obj = qpdf.new_stream();
    assert!(obj.is_stream());
    assert_eq!(obj.to_string(), "1 0 R");

    let stream_dict = obj.get_stream_dictionary();
    stream_dict.set("/Type", &qpdf.new_name("/Stream"));

    let indirect = obj.make_indirect();
    assert!(indirect.is_indirect());
    assert_ne!(indirect.get_id(), 0);
    assert_eq!(indirect.get_generation(), 0);
    assert_eq!(indirect.to_string(), "2 0 R");
}

#[test]
fn test_parse_object() {
    let text = "<< /Type /Page /Resources << /XObject null >> /MediaBox null /Contents null >>";
    let qpdf = Qpdf::new();
    let obj = qpdf.parse_object(text).unwrap();
    assert!(obj.is_dictionary());
    println!("{}", obj.to_string());
    println!("version: {}", qpdf.get_pdf_version());
}

#[test]
fn test_error() {
    let text = "<<--< /Type -- null >>";
    let qpdf = Qpdf::new();
    qpdf.get_page(0);
    let result = qpdf.parse_object(text);
    assert!(result.is_err());
    println!("{:?}", result);

    let trailer = qpdf.get_trailer();
    assert!(trailer.is_none());

    let root = qpdf.get_root();
    assert!(root.is_none());

    let obj = qpdf.get_object_by_id(1234, 1);
    assert!(obj.is_none());
}

#[test]
fn test_array() {
    let qpdf = Qpdf::new();
    let mut arr = qpdf.new_array();
    arr.push(&qpdf.new_integer(1));
    arr.push(&qpdf.new_integer(2));
    arr.push(&qpdf.new_integer(3));
    assert_eq!(arr.inner().to_string(), "[ 1 2 3 ]");

    assert!(arr.get(10).is_none());

    assert_eq!(
        arr.iter().map(|v| v.as_i32()).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    arr.set(1, &qpdf.new_integer(5));
    assert_eq!(arr.inner().to_string(), "[ 1 5 3 ]");
}

#[test]
fn test_dictionary() {
    let qpdf = Qpdf::new();
    let dict: QpdfDictionary = qpdf
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

    assert!(dict.get("/Type").unwrap().is_name());
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
    let qpdf = Qpdf::new();
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
    let pages = qpdf.get_pages().unwrap();
    assert_eq!(pages.len(), 2);

    for page in pages {
        let dict: QpdfDictionary = page.into();
        let keys = dict.keys();
        assert!(!keys.is_empty());
        println!("{:?}", keys);

        let data = dict.inner.get_page_content_data().unwrap();
        println!("{}", String::from_utf8_lossy(data.as_ref()));

        qpdf.add_page(&dict.inner.clone(), false).unwrap();
    }

    let buffer = qpdf.save_to_memory().unwrap();
    let saved_pdf = Qpdf::load_from_memory(&buffer).unwrap();
    assert_eq!(saved_pdf.get_num_pages().unwrap(), 4);

    let pages = saved_pdf.get_pages().unwrap();
    for page in pages {
        saved_pdf.remove_page(&page).unwrap();
    }
    assert_eq!(saved_pdf.get_num_pages().unwrap(), 0);
}

#[test]
fn test_pdf_encrypted() {
    let qpdf = Qpdf::load("tests/data/encrypted.pdf");
    assert!(qpdf.is_err());
    println!("{:?}", qpdf);

    let qpdf = Qpdf::load_encrypted("tests/data/encrypted.pdf", "test");
    assert!(qpdf.is_ok());

    let data = std::fs::read("tests/data/encrypted.pdf").unwrap();
    let qpdf = Qpdf::load_from_memory_encrypted(&data, "test");
    assert!(qpdf.is_ok());
}
