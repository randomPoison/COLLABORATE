extern crate collaborate;

use ::collaborate::*;

#[test]
fn no_xml_decl() {
    static DOCUMENT: &'static str = r#"
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let _ = VersionedDocument::from_str(DOCUMENT).unwrap();
}

#[test]
fn doctype() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <!DOCTYPE note SYSTEM "Note.dtd">
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let _ = VersionedDocument::from_str(DOCUMENT).unwrap();
}

#[test]
fn extra_whitespace() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>

    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">

        <asset        >
            <created>    2017-02-07T20:44:30Z        </created       >
            <modified    > 2017-02-07T20:44:30Z             </modified      >
        </asset>

    </COLLADA      >

    "#;

    let _ = VersionedDocument::from_str(DOCUMENT).unwrap();
}

#[test]
fn default_attrib_value() {
    use ::collaborate::v1_4::*;

    static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");

    let source = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    let document = Collada::from_str(&*source).unwrap();
    let library = document.libraries[5].as_library_geometries().unwrap();
    let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
    let source = &mesh.sources[0];
    let array = source.array.as_ref().and_then(Array::as_float_array).unwrap();

    // Verify that the `[optional_with_default = "XXX"]` attribute is working correctly. We know
    // that the document doesn't declare the "digits" or "magnitude" parameter on this
    // `<float_array>` element, so we check to see if they end up with the right default values.
    assert_eq!(6, array.digits, "Default value for `FloatArray::digits` should be 6");
    assert_eq!(38, array.magnitude, "Default value for `FloatArray::magnitude` should be 38");

}

#[test]
fn float_array_text_contents() {
    use ::collaborate::v1_4::*;

    static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");
    static EXPECTED: &'static [f32] = &[1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -0.9999998, -1.0, -0.9999997, 1.0, -1.0, 1.0, 0.9999995, 1.0, 0.9999994, -1.000001, 1.0, -1.0, -0.9999997, 1.0, -1.0, 1.0, 1.0];

    let source = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    let document = Collada::from_str(&*source).unwrap();
    let library = document.libraries[5].as_library_geometries().unwrap();
    let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
    let source = &mesh.sources[0];
    let array = source.array.as_ref().and_then(Array::as_float_array).unwrap();

    assert_eq!(EXPECTED, &*array.data, "`<float_array>` contents were not parsed correctly");
}
