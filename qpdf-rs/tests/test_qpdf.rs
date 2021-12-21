use std::collections::HashSet;

use qpdf::*;

#[test]
fn test_qpdf_version() {
    println!("{}", Qpdf::library_version());
}

#[test]
fn test_qpdf_new_objects() {
    let qpdf = Qpdf::new();
    let obj = qpdf.new_bool(true);
    assert!(obj.is_bool() && obj.as_bool());

    let obj = qpdf.new_name("foo");
    assert!(obj.is_name() && obj.as_name() == "foo");

    let obj = qpdf.new_integer(12_3456_7890);
    assert!(obj.is_scalar() && obj.as_i64() == 12_3456_7890);

    let obj = qpdf.new_null();
    assert!(obj.is_null());

    let obj = qpdf.new_real(1.2345, 3);
    assert_eq!(obj.as_real(), "1.234");

    let obj = qpdf.new_uninitialized();
    assert!(!obj.is_initialized());

    let obj = qpdf.new_stream();
    assert!(obj.is_stream());
}

#[test]
fn test_parse_object() {
    let qpdf = Qpdf::new();
    let obj = qpdf
        .parse_object(
            "<< /Type /Page /Resources << /XObject null >> /MediaBox null /Contents null >>",
        )
        .unwrap();
    assert!(obj.is_dictionary());
    println!("{}", obj.to_string());
}

#[test]
fn test_array() {
    let qpdf = Qpdf::new();
    let mut arr = qpdf.new_array();
    arr.push(qpdf.new_integer(1));
    arr.push(qpdf.new_integer(2));
    arr.push(qpdf.new_integer(3));
    assert_eq!(arr.inner.to_string(), "[ 1 2 3 ]");

    assert!(arr.get(10).is_none());

    assert_eq!(
        arr.iter().map(|v| v.as_i32()).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    arr.set(1, qpdf.new_integer(5));
    assert_eq!(arr.inner.to_string(), "[ 1 5 3 ]");
}

#[test]
fn test_dictionary() {
    let qpdf = Qpdf::new();
    let dict = qpdf
        .parse_object("<< /Type /Page /Resources << /XObject null >> /MediaBox [1 2 3 4] /Contents (hello) >>")
        .unwrap()
        .into_dictionary();

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

    dict.set("/MyKey", qpdf.new_bool(true));
    assert!(dict.get("/MyKey").unwrap().as_bool());

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
}

fn load_pdf() -> Qpdf {
    let qpdf = Qpdf::load("tests/data/test.pdf").unwrap();
    qpdf
}

#[test]
fn test_pdf_ops() {
    let qpdf = load_pdf();
    println!("{:?}", qpdf.get_pdf_version().unwrap());
    let pages = qpdf.get_pages().unwrap();
    assert_eq!(pages.len(), 2);

    for page in pages {
        let dict = page.into_dictionary();
        let keys = dict.keys();
        assert!(!keys.is_empty());
        println!("{:?}", keys);

        let data = dict.inner.get_page_content_data().unwrap();
        println!("{}", String::from_utf8_lossy(data.as_ref()));

        qpdf.add_page(&dict.inner.clone(), false).unwrap();
    }

    let buffer = qpdf.save_to_memory().unwrap();
    let saved_pdf = Qpdf::do_load_memory(buffer, None).unwrap();
    assert_eq!(saved_pdf.get_num_pages().unwrap(), 4);

    let pages = saved_pdf.get_pages().unwrap();
    for page in pages {
        saved_pdf.remove_page(&page).unwrap();
    }
    assert_eq!(saved_pdf.get_num_pages().unwrap(), 0);
}
