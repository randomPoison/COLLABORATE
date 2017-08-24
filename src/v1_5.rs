use {AnyUri, DateTime, Extra, Result, Error, ErrorKind, Unit, UpAxis, utils};
use std::io::Read;
use utils::*;
use xml::common::Position;
use xml::reader::EventReader;
use xml::reader::XmlEvent::*;

/// The logic behind parsing the COLLADA document.
///
/// `from_str()` and `read()` just create the `xml::EventReader` and then defer to `parse()`.
///
/// TODO: This is currently publicly exported. That shouldn't happen.
pub fn collaborate<R: Read>(mut reader: EventReader<R>, version: String, base: Option<AnyUri>) -> Result<Collada> {
    // The next event must be the `<asset>` tag. No text data is allowed, and
    // whitespace/comments aren't emitted.
    let start_element = utils::required_start_element(&mut reader, "COLLADA", "asset")?;
    let asset = Asset::parse_element(&mut reader, start_element)?;

    // Eat any events until we get to the `</COLLADA>` tag.
    // TODO: Actually parse the body of the document.
    loop {
        match reader.next()? {
            EndElement { ref name } if name.local_name == "COLLADA" => { break }
            _ => {}
        }
    }

    // TODO: Verify the next event is the `EndDocument` event.
    match reader.next()? {
        EndDocument => {}

        // Same logic here as with the starting event. The only things that can come after the
        // close tag are comments, white space, and processing instructions, all of which we
        // ignore. This can change with future versions of xml-rs, though.
        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }

    Ok(Collada {
        version: version,
        asset: asset,
        base_uri: base,
    })
}

/// Represents a parsed COLLADA document.
#[derive(Debug, Clone, PartialEq)]
pub struct Collada {
    /// The version string for the COLLADA specification used by the document.
    ///
    /// Only "1.4.0", "1.4.1", and "1.5.0" are supported currently.
    pub version: String,

    /// The base uri for any relative URIs in the document.
    ///
    /// Specified by the `base` attribute on the root `<COLLADA>` element.
    pub base_uri: Option<AnyUri>,

    /// Global metadata about the COLLADA document.
    pub asset: Asset,
}

impl Collada {
    /// Read a COLLADA document from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use collaborate::Collada;
    ///
    /// static DOCUMENT: &'static str = r#"
    ///     <?xml version="1.0" encoding="utf-8"?>
    ///     <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
    ///         <asset>
    ///             <created>2017-02-07T20:44:30Z</created>
    ///             <modified>2017-02-07T20:44:30Z</modified>
    ///         </asset>
    ///     </COLLADA>
    /// "#;
    ///
    /// let collada = Collada::from_str(DOCUMENT).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation][crate].
    ///
    /// [crate]: index.html
    pub fn from_str(source: &str) -> Result<Collada> {
        let reader = EventReader::new_with_config(source.as_bytes(), utils::PARSER_CONFIG.clone());
        utils::parse(reader)
    }

    /// Attempts to parse the contents of a COLLADA document.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use std::fs::File;
    /// use collaborate::Collada;
    ///
    /// let file = File::open("resources/blender_cube.dae").unwrap();
    /// let collada = Collada::read(file).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation][crate].
    ///
    /// [crate]: index.html
    pub fn read<R: Read>(reader: R) -> Result<Collada> {
        let reader = EventReader::new_with_config(reader, utils::PARSER_CONFIG.clone());
        utils::parse(reader)
    }
}

/// Asset-management information about an element.
///
/// Includes both asset metadata, such as a list of contributors and keywords, as well
/// as functional information, such as units of distance and the up axis for the asset.
///
/// # COLLADA Versions
///
/// `coverage` and `extras` were added in COLLADA version `1.5.0`.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "asset"]
pub struct Asset {
    /// The list of contributors who worked on the asset.
    #[child]
    pub contributors: Vec<Contributor>,

    /// Specifies the location of the visual scene in physical space.
    #[child]
    pub coverage: Option<Coverage>,

    /// Specifies the date and time that the asset was created.
    #[child]
    pub created: DateTime,

    /// A list of keywords used as search criteria for the asset.
    #[child]
    pub keywords: Option<String>,

    /// Contains the date and time that the parent element was last modified.
    #[child]
    pub modified: DateTime,

    /// Contains revision information about the asset.
    ///
    /// This field is free-form, with no formatting required by the COLLADA specification.
    #[child]
    pub revision: Option<String>,

    /// Contains a description of the topical subject of the asset.
    ///
    /// This field is free-form, with no formatting required by the COLLADA specification.
    #[child]
    pub subject: Option<String>,

    /// Contains title information for the asset.
    ///
    /// This field is free-form, with no formatting required by the COLLADA specification.
    #[child]
    pub title: Option<String>,

    /// Defines the unit of distance for this asset.
    ///
    /// This unit is used by the asset and all of its children, unless overridden by a more
    /// local `Unit`.
    #[child]
    #[optional_with_default]
    pub unit: Unit,

    /// Describes the coordinate system of the asset.
    ///
    /// See the documentation for [`UpAxis`] for more details.
    ///
    /// [`UpAxis`]: ../struct.UpAxis.html
    #[child]
    #[optional_with_default]
    pub up_axis: UpAxis,

    /// Provides arbitrary additional data about the asset.
    ///
    /// See the [`Extra`] documentation for more information.
    ///
    /// [`Extra`]: ./struct.Extra.html
    #[child]
    pub extras: Vec<Extra>,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "coverage"]
pub struct Coverage {
    #[child]
    pub geographic_location: Option<GeographicLocation>,
}

/// Information about a contributor to an asset.
///
/// Contributor data is largely free-form text data meant to informally describe either the author
/// or the author's work on the asset. The exceptions are `author_email`, `author_website`, and
/// `source_data`, which are strictly formatted data (be it a URI or email address).
///
/// # COLLADA Versions
///
/// `author_email` and `author_website` were added in COLLADA version `1.5.0`.
#[derive(Debug, Clone, Default, PartialEq, Eq, ColladaElement)]
#[name = "contributor"]
pub struct Contributor {
    /// The author's name, if present.
    #[child]
    pub author: Option<String>,

    /// The author's full email address, if present.
    // TODO: Should we use some `Email` type? The 1.5.0 COLLADA spec provides an RFC defining the
    // exact format this data follows (I assume it's just the RFC that defines valid email
    // addresses).
    #[child]
    pub author_email: Option<String>,

    /// The URL for the author's website, if present.
    #[child]
    #[text_data]
    pub author_website: Option<AnyUri>,

    /// The name of the authoring tool.
    #[child]
    pub authoring_tool: Option<String>,

    /// Free-form comments from the author.
    #[child]
    pub comments: Option<String>,

    /// Copyright information about the asset. Does not adhere to a formatting standard.
    #[child]
    pub copyright: Option<String>,

    /// A URI reference to the source data for the asset.
    ///
    /// For example, if the asset based off a file `tank.s3d`, the value might be
    /// `c:/models/tank.s3d`.
    #[child]
    #[text_data]
    pub source_data: Option<AnyUri>,
}

/// Defines geographic location information for an [`Asset`][Asset].
///
/// A geographic location is given in latitude, longitude, and altitude coordinates as defined by
/// [WGS 84][WGS 84] world geodetic system.
///
/// [Asset]: struct.Asset.html
/// [WGS 84]: https://en.wikipedia.org/wiki/World_Geodetic_System#A_new_World_Geodetic_System:_WGS_84
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "geographic_location"]
pub struct GeographicLocation {
    /// The longitude of the location. Will be in the range -180.0 to 180.0.
    #[child]
    #[text_data]
    pub longitude: f64,

    /// The latitude of the location. Will be in the range -180.0 to 180.0.
    #[child]
    #[text_data]
    pub latitude: f64,

    /// Specifies the altitude, either relative to global sea level or relative to ground level.
    #[child]
    pub altitude: Altitude,
}

/// Specifies the altitude of a [`GeographicLocation`][GeographicLocation].
///
/// [GeographicLocation]: struct.GeographicLocation.html
#[derive(Debug, Clone, PartialEq)]
pub enum Altitude {
    /// The altitude is relative to global sea level.
    Absolute(f64),

    /// The altitude is relative to ground level at the specified latitude and longitude.
    RelativeToGround(f64),
}

impl ColladaElement for Altitude {
    fn name_test(name: &str) -> bool {
        name == "altitude"
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<Self>
    where
        R: Read,
    {
        let mut mode = None;
        for attribute in element_start.attributes {
            match &*attribute.name.local_name {
                "mode" => {
                    mode = Some(attribute.value);
                }

                attrib_name @ _ => {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedAttribute {
                            element: "altitude",
                            attribute: attrib_name.into(),
                            expected: vec!["mode"],
                        },
                    });
                }
            }
        }

        let mode = match mode {
            Some(mode) => { mode }
            None => {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::MissingAttribute {
                        element: "altitude",
                        attribute: "mode",
                    },
                });
            }
        };

        match &*mode {
            "absolute" => {
                let value = utils::required_text_contents(reader, "altitude")?;
                Ok(Altitude::Absolute(value))
            }

            "relativeToGround" => {
                let value = utils::required_text_contents(reader, "altitude")?;
                Ok(Altitude::RelativeToGround(value))
            }

            _ => {
                Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::InvalidValue {
                        element: "altitude",
                        value: mode,
                    },
                })
            }
        }
    }

    fn add_names(names: &mut Vec<&'static str>) {
        names.push("altitude");
    }
}
