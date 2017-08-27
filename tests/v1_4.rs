extern crate collaborate;

use ::collaborate::*;
use ::collaborate::common::*;
use ::collaborate::v1_4::*;

#[test]
fn blender_cube() {
    static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");

    let document = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    let _ = Collada::from_str(&*document).unwrap();
}

#[test]
fn collada_asset_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;


    let expected = Collada {
        version: "1.4.1".into(),
        xmlns: None,
        base_uri: None,
        asset: Asset {
            contributors: vec![],
            created: "2017-02-07T20:44:30Z".parse().unwrap(),
            keywords: None,
            modified: "2017-02-07T20:44:30Z".parse().unwrap(),
            revision: None,
            subject: None,
            title: None,
            unit: Unit {
                meter: 1.0,
                name: "meter".into(),
            },
            up_axis: UpAxis::Y,
        },
        libraries: Vec::new(),
        scene: None,
        extras: Vec::new(),
    };

    let actual = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn collada_missing_asset() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 2, column: 4 },
        kind: ErrorKind::MissingElement {
            parent: "COLLADA".into(),
            expected: vec!["asset"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn asset_full() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor />
            <contributor />
            <contributor />
            <created>2017-02-07T20:44:30Z</created>
            <keywords>foo bar baz</keywords>
            <modified>2017-02-07T20:44:30Z</modified>
            <revision>7</revision>
            <subject>A thing</subject>
            <title>Model of a thing</title>
            <unit meter="7" name="septimeter" />
            <up_axis>Z_UP</up_axis>
        </asset>
    </COLLADA>
    "#;

    let expected = Asset {
        contributors: vec![Contributor::default(), Contributor::default(), Contributor::default()],
        created: "2017-02-07T20:44:30Z".parse().unwrap(),
        keywords: Some("foo bar baz".into()),
        modified: "2017-02-07T20:44:30Z".parse().unwrap(),
        revision: Some("7".into()),
        subject: Some("A thing".into()),
        title: Some("Model of a thing".into()),
        unit: Unit {
            meter: 7.0,
            name: "septimeter".into(),
        },
        up_axis: UpAxis::Z,
    };

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, collada.asset);
}

#[test]
fn asset_blender() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>Blender User</author>
                <authoring_tool>Blender 2.78.0 commit date:2016-10-24, commit time:12:20, hash:e8299c8</authoring_tool>
            </contributor>
            <created>2017-02-01T09:29:54</created>
            <modified>2017-02-01T09:29:54</modified>
            <unit name="meter" meter="1"/>
            <up_axis>Z_UP</up_axis>
        </asset>
    </COLLADA>
    "#;

    let expected = Asset {
        contributors: vec![
            Contributor {
                author: Some("Blender User".into()),
                authoring_tool: Some("Blender 2.78.0 commit date:2016-10-24, commit time:12:20, hash:e8299c8".into()),
                .. Contributor::default()
            },
        ],
        created: "2017-02-01T09:29:54".parse().unwrap(),
        keywords: None,
        modified: "2017-02-01T09:29:54".parse().unwrap(),
        revision: None,
        subject: None,
        title: None,
        unit: Unit {
            meter: 1.0,
            name: "meter".into(),
        },
        up_axis: UpAxis::Z,
    };

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, collada.asset);
}

#[test]
fn asset_wrong_version() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor />
            <contributor />
            <contributor />
            <coverage>
                <geographic_location>
                    <longitude>-105.2830</longitude>
                    <latitude>40.0170</latitude>
                    <altitude mode="relativeToGround">0</altitude>
                </geographic_location>
            </coverage>
            <created>2017-02-07T20:44:30Z</created>
            <keywords>foo bar baz</keywords>
            <modified>2017-02-07T20:44:30Z</modified>
            <revision>7</revision>
            <subject>A thing</subject>
            <title>Model of a thing</title>
            <unit meter="7" name="septimeter" />
            <up_axis>Z_UP</up_axis>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 7, column: 12 },
        kind: ErrorKind::UnexpectedElement {
            parent: "asset",
            element: "coverage".into(),
            expected: vec!["contributor", "created", "keywords", "modified", "revision", "subject", "title", "unit", "up_axis"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor />
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(vec![Contributor::default()], collada.asset.contributors);
}

#[test]
fn contributor_full() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments>This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Contributor {
        author: Some("David LeGare".into()),
        authoring_tool: Some("Atom".into()),
        comments: Some("This is a sample COLLADA document.".into()),
        copyright: Some("David LeGare, free for public use".into()),
        source_data: Some("C:/models/tank.s3d".parse().unwrap()),
    };

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(vec![expected], collada.asset.contributors);
}

#[test]
fn contributor_wrong_order() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <comments>This is a sample COLLADA document.</comments>
                <authoring_tool>Atom</authoring_tool>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 7, column: 16 },
        kind: ErrorKind::UnexpectedElement {
            parent: "contributor".into(),
            element: "authoring_tool".into(),
            expected: vec!["author", "authoring_tool", "comments", "copyright", "source_data"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_illegal_child() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments>This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
                <foo>Some foo data</foo>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 10, column: 16 },
        kind: ErrorKind::UnexpectedElement {
            parent: "contributor".into(),
            element: "foo".into(),
            expected: vec!["author", "authoring_tool", "comments", "copyright", "source_data"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_wrong_version() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <author_email>dl@email.com</author_email>
                <authoring_tool>Atom</authoring_tool>
                <comments>This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 6, column: 16 },
        kind: ErrorKind::UnexpectedElement {
            parent: "contributor".into(),
            element: "author_email".into(),
            expected: vec!["author", "authoring_tool", "comments", "copyright", "source_data"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_illegal_attribute() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor foo="bar">
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments>This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 4, column: 12 },
        kind: ErrorKind::UnexpectedAttribute {
            element: "contributor".into(),
            attribute: "foo".into(),
            expected: vec![],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_illegal_child_attribute() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments foo="bar">This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 7, column: 16 },
        kind: ErrorKind::UnexpectedAttribute {
            element: "comments".into(),
            attribute: "foo".into(),
            expected: vec![],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn polylist_iter() {
    use ::collaborate::v1_4::*;

    static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");

    let source = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    let document = Collada::from_str(&*source).unwrap();
    let library = document.libraries[5].as_library_geometries().unwrap();
    let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
    let polylist = mesh.primitives[0].as_polylist().unwrap();

    let mut polygons = polylist.iter();

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([0, 0].as_ref()), vertices.next());
        assert_eq!(Some([2, 0].as_ref()), vertices.next());
        assert_eq!(Some([3, 0].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([7, 1].as_ref()), vertices.next());
        assert_eq!(Some([5, 1].as_ref()), vertices.next());
        assert_eq!(Some([4, 1].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([4, 2].as_ref()), vertices.next());
        assert_eq!(Some([1, 2].as_ref()), vertices.next());
        assert_eq!(Some([0, 2].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([5, 3].as_ref()), vertices.next());
        assert_eq!(Some([2, 3].as_ref()), vertices.next());
        assert_eq!(Some([1, 3].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([2, 4].as_ref()), vertices.next());
        assert_eq!(Some([7, 4].as_ref()), vertices.next());
        assert_eq!(Some([3, 4].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([0, 5].as_ref()), vertices.next());
        assert_eq!(Some([7, 5].as_ref()), vertices.next());
        assert_eq!(Some([4, 5].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([0, 6].as_ref()), vertices.next());
        assert_eq!(Some([1, 6].as_ref()), vertices.next());
        assert_eq!(Some([2, 6].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([7, 7].as_ref()), vertices.next());
        assert_eq!(Some([6, 7].as_ref()), vertices.next());
        assert_eq!(Some([5, 7].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([4, 8].as_ref()), vertices.next());
        assert_eq!(Some([5, 8].as_ref()), vertices.next());
        assert_eq!(Some([1, 8].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([5, 9].as_ref()), vertices.next());
        assert_eq!(Some([6, 9].as_ref()), vertices.next());
        assert_eq!(Some([2, 9].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([2, 10].as_ref()), vertices.next());
        assert_eq!(Some([6, 10].as_ref()), vertices.next());
        assert_eq!(Some([7, 10].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    {
        let polygon = polygons.next().unwrap();
        assert_eq!(3, polygon.len());

        let mut vertices = polygon.vertices();
        assert_eq!(Some([0, 11].as_ref()), vertices.next());
        assert_eq!(Some([3, 11].as_ref()), vertices.next());
        assert_eq!(Some([7, 11].as_ref()), vertices.next());
        assert_eq!(None, vertices.next());
    }

    assert!(polygons.next().is_none());
}
