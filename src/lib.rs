//! A library for parsing and processing COLLADA documents.
//!
//! > NOTE: Currently this library is focussed on supporting the `1.4.1` COLLADA specification.
//! > Support for version `1.5.0` is desired, but not a current priority. If you'd like to help
//! > add support, please feel free to open issues in the issue tracker or make a pull request.
//!
//! [COLLADA] is a COLLAborative Design Activity that defines an XML-based schema to
//! enable 3D authoring applications to freely exchange digital assets. It supports a vast array of
//! features used in 3D modeling, animation, and VFX work, and provides an open, non-proprietary
//! alternative to common formats like [FBX].
//!
//! This provides functionality for parsing a COLLADA document and utilities for processing the
//! contained data, with the intention of enable direct usage of COLLADA data as well as
//! interchange of document data into other formats.
//!
//! # Quick Start
//!
//! The easiest way to parse a COLLADA document is to load it from a file and use
//! [`VersionedDocument::read`]:
//!
//! ```
//! # #![allow(unused_variables)]
//! use std::fs::File;
//! use collaborate::VersionedDocument;
//!
//! let file = File::open("resources/blender_cube.dae").unwrap();
//! match VersionedDocument::read(file).unwrap() {
//!     VersionedDocument::V1_4(document) => {
//!         println!("Loaded a 1.4.1 document: {:?}", document);
//!     }
//!
//!     VersionedDocument::V1_5(document) => {
//!         println!("Loaded a 1.5.0 document: {:?}", document);
//!     }
//! }
//! ```
//!
//! Each variant wraps a `Collada` object which provides direct access to all data in the
//! document, directly recreating the logical structure of the document as a Rust type.
//! [`VersionedDocument`] will also automatically detect which version of the COLLADA schema the
//! document uses, and will parse it correctly.
//!
//! If you know ahead of time which version of the COLLADA spec you'll be using, you can use
//! [`v1_4::Collada`] or [`v1_5::Collada`] directly for convenience:
//!
//! ```
//! # #![allow(unused_variables)]
//! use std::fs::File;
//! use collaborate::v1_4;
//!
//! let file = File::open("resources/blender_cube.dae").unwrap();
//!
//! // I already know that `blender_cube.dae` uses COLLADA version 1.4.1.
//! let document = v1_4::Collada::read(file).unwrap();
//! ```
//!
//! # COLLADA Versions
//!
//! Currently there are 3 COLLADA versions supported by this library: `1.4.0`, `1.4.1`, and
//! `1.5.0`. Version `1.4.0` documents are automatically handled like `1.4.1`, so users of this
//! library never need to worry about the distinction. Version `1.5` is not compatible with
//! version `1.4`, so the two are handled separately. As such, `1.4` documents are represented
//! by [`v1_4::Collada`] and the types in the [`v1_4`] module, and `1.5` documents are represented
//! by [`v1_5::Collada`] and the types in the [`v1_5`] module.
//!
//! Do avoid having to know the version of the document before you load it, you can use
//! [`VersionedDocument`] to detect the version and parse the document into the correct type.
//! COLLABORATE makes no effort to unify incompatible versions of the specification, so users of
//! COLLABORATE will have to handle both versions separately if they wish to do so.
//!
//! # 3rd Party Extensions
//!
//! The COLLADA format allows for semi-arbitrary extensions to the standard, allowing applications
//! to include application-specific data. This extra data is considered "optional", but may allow
//! applications consuming the COLLADA document to more accurately recreate the scene contained
//! in the document. This library attempts to directly support common 3rd party extensions,
//! primarily those for Blender and Maya. In the case that the 3rd party extension is not
//! directly supported, the underlying XML will be preserved so that the client code can attempt
//! to still use the data.
//!
//! [COLLADA]: https://www.khronos.org/collada/
//! [FBX]: https://en.wikipedia.org/wiki/FBX
//! [`VersionedDocument`]: ./enum.VersionedDocument.html
//! [`VersionedDocument::read`]: ./enum.VersionedDocument.html#method.read
//! [`v1_4`]: ./v1_4/index.html
//! [`v1_5`]: ./v1_5/index.html
//! [`v1_4::Collada`]: ./v1_4/struct.Collada.html
//! [`v1_5::Collada`]: ./v1_5/struct.Collada.html

pub extern crate chrono;
#[macro_use]
extern crate collaborate_derive;
extern crate xml;

pub use xml::common::TextPosition;
pub use xml::reader::{Error as XmlError, XmlEvent};

use common::UriFragmentParseError;
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::num::{ParseFloatError, ParseIntError};
use utils::{ColladaElement, StringListDisplay};
use xml::common::Position;
use xml::reader::EventReader;

pub mod common;
pub mod v1_4;
pub mod v1_5;

mod utils;

/// A helper type for parsing documents without knowing the version ahead of time.
///
/// If you know the specification used by a COLLADA document ahead of time, you can use
/// [`v1_4::Collada`] to load `1.4.0` and `1.4.1` documents, and [`v1_5::Collada`] to load `1.5.0`
/// documents. If, on the other hand, you don't know what version the document uses, then you
/// can use `VersionedDocument` to detect which version to use.
///
/// # Examples
///
/// ```
/// # #![allow(unused_variables)]
/// use std::fs::File;
/// use collaborate::VersionedDocument;
///
/// let file = File::open("resources/blender_cube.dae").unwrap();
/// match VersionedDocument::read(file).unwrap() {
///     VersionedDocument::V1_4(document) => {
///         println!("Loaded a 1.4.1 document: {:?}", document);
///     }
///
///     VersionedDocument::V1_5(document) => {
///         println!("Loaded a 1.5.0 document: {:?}", document);
///     }
/// }
/// ```
///
/// [`v1_4::Collada`]: ./v1_4/struct.Collada.html
/// [`v1_5::Collada`]: ./v1_5/struct.Collada.html
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum VersionedDocument {
    /// A `1.4.0` or `1.4.1` document.
    V1_4(v1_4::Collada),

    /// A `1.5.0` document.
    V1_5(v1_5::Collada),
}

impl VersionedDocument {
    /// Read a COLLADA document from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use collaborate::VersionedDocument;
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
    /// match VersionedDocument::from_str(DOCUMENT).unwrap() {
    ///     VersionedDocument::V1_4(document) => {
    ///         println!("Document contents: {:?}", document);
    ///     }
    ///
    ///     _ => panic!("Impossible, the document version was 1.4.1"),
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation](./index.html).
    pub fn from_str(source: &str) -> Result<VersionedDocument> {
        let reader = EventReader::new_with_config(source.as_bytes(), utils::PARSER_CONFIG.clone());
        Self::parse(reader)
    }

    /// Attempts to parse the contents of a COLLADA document.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use std::fs::File;
    /// use collaborate::VersionedDocument;
    ///
    /// let file = File::open("resources/blender_cube.dae").unwrap();
    /// match VersionedDocument::read(file).unwrap() {
    ///     VersionedDocument::V1_4(document) => {
    ///         println!("Loaded a 1.4.1 document: {:?}", document);
    ///     }
    ///
    ///     VersionedDocument::V1_5(document) => {
    ///         println!("Loaded a 1.5.0 document: {:?}", document);
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation][crate].
    ///
    /// [crate]: index.html
    pub fn read<R: Read>(reader: R) -> Result<VersionedDocument> {
        let reader = EventReader::new_with_config(reader, utils::PARSER_CONFIG.clone());
        Self::parse(reader)
    }

    pub fn parse<R: Read>(mut reader: EventReader<R>) -> Result<VersionedDocument> {
        // Get the opening `<COLLADA>` tag and find the "version" attribute.
        let element_start = utils::get_document_start(&mut reader)?;
        let version = element_start.attributes.iter()
            .find(|attrib| attrib.name.local_name == "version")
            .map(|attrib| attrib.value.clone())
            .ok_or(Error {
                position: reader.position(),
                kind: ErrorKind::MissingAttribute {
                    element: "COLLADA",
                    attribute: "version",
                },
            })?;

        match &*version {
            "1.4.0" | "1.4.1" => {
                v1_4::Collada::parse_element(&mut reader, element_start).map(Into::into)
            }

            "1.5.0" => {
                v1_5::Collada::parse_element(&mut reader, element_start).map(Into::into)
            }

            _ => {
                Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::UnsupportedVersion {
                        version: version,
                    },
                })
            }
        }
    }
}

impl From<v1_4::Collada> for VersionedDocument {
    fn from(from: v1_4::Collada) -> VersionedDocument {
        VersionedDocument::V1_4(from)
    }
}

impl From<v1_5::Collada> for VersionedDocument {
    fn from(from: v1_5::Collada) -> VersionedDocument {
        VersionedDocument::V1_5(from)
    }
}

/// A COLLADA parsing error.
///
/// Contains where in the document the error occurred (i.e. line number and column), and
/// details about the nature of the error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub position: TextPosition,
    pub kind: ErrorKind,
}

impl From<xml::reader::Error> for Error {
    fn from(from: xml::reader::Error) -> Error {
        Error {
            position: from.position(),
            kind: ErrorKind::XmlError(from),
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        write!(formatter, "Error at {}: {}", self.position, self.kind)
    }
}

/// The specific error variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// An element was missing a required attribute.
    ///
    /// Some elements in the COLLADA specification have required attributes. If such a requried
    /// attribute is missing, then this error is returned.
    MissingAttribute {
        /// The element that was missing an attribute.
        element: &'static str,

        /// The attribute that expected to be present.
        attribute: &'static str,
    },

    /// A required elent was missing.
    ///
    /// Some elements in the COLLADA document have required children, or require that at least one
    /// of a set of children are present. If such a required element is missing, this error is
    /// returned.
    MissingElement {
        /// The element that was expecting a child element.
        parent: &'static str,

        /// The set of required child elements.
        ///
        /// If there is only one expected child then it is a required child. If there are multiple
        /// expected children then at least one of them is required.
        expected: Vec<&'static str>,
    },

    /// An element was missing required text data.
    ///
    /// Some elements in the COLLADA document are required to contain some kind of data. If such
    /// an element is missing any required data, this error is returned.
    MissingValue {
        element: &'static str,
    },

    /// A floating point value was formatted incorrectly.
    ///
    /// Floating point values are parsed according to Rust's [standard handling for floating point
    /// numbers][f64::from_str].
    ///
    /// [f64::from_str]: https://doc.rust-lang.org/std/primitive.f64.html#method.from_str
    ParseFloatError(ParseFloatError),

    /// A integer value was formatted incorrectly.
    ///
    /// Floating point values are parsed according to Rust's [standard handling for integers](https://doc.rust-lang.org/std/primitive.usize.html#method.from_str).
    ParseIntError(ParseIntError),

    /// A datetime string was formatted incorrectly.
    ///
    /// Datetime strings in COLLADA are in the [ISO 8601][ISO 8601] format, and improperly
    /// formatted datetime values will cause this error to be returned.
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    TimeError(chrono::ParseError),

    /// An element had an attribute that isn't allowed.
    ///
    /// Elements in a COLLADA document are restricted to having only specific attributes. The
    /// presence of an attribute that's not part of the COLLADA specification will cause this
    /// error to be returned.
    UnexpectedAttribute {
        /// The element that had the unexpected attribute.
        element: &'static str,

        /// The unexpected attribute.
        attribute: String,

        /// The set of attributes allowed for this element.
        expected: Vec<&'static str>,
    },

    /// An element contained non-markup text that isn't allowed.
    ///
    /// Most elements may only have other tags as children, only a small subset of COLLADA
    /// elements contain actual data. If an element that only is allowed to have children contains
    /// text data it is considered an error.
    UnexpectedCharacterData {
        /// The element that contained the unexpected text data.
        element: &'static str,

        /// The data that was found.
        ///
        /// The `Display` message for this error does not include the value of `data` as it is
        /// often not relevant to end users, who can often go and check the original COLLADA
        /// document if they wish to know what the erroneous text was. It is preserved in the
        /// error object to assist in debugging.
        data: String,
    },

    /// An element had a child element that isn't allowed.
    ///
    /// The COLLADA specification determines what children an element may have, as well as what
    /// order those children may appear in. If an element has a child that is not allowed, or an
    /// allowed child appears out of order, then this error is returned.
    UnexpectedElement {
        /// The element that had the unexpected child.
        parent: &'static str,

        /// The element that is not allowed or is out of order.
        element: String,

        /// The set of expected child elements for `parent`.
        ///
        /// If `element` is in `expected` then it means the element is a valid child but appeared
        /// out of order.
        expected: Vec<&'static str>,
    },

    /// The document started with an element other than `<COLLADA>`.
    ///
    /// The only valid root element for a COLLADA document is the `<COLLADA>` element. This is
    /// consistent across all supported versions of the COLLADA specificaiton. Any other root
    /// element returns this error.
    ///
    /// The presence of an invalid root element will generally indicate that a non-COLLADA
    /// document was accidentally passed to the parser. Double check that you are using the
    /// intended document.
    UnexpectedRootElement {
        /// The element that appeared at the root of the document.
        element: String,
    },

    /// An element or attribute contained text data that was formatted incorrectly.
    InvalidValue {
        element: &'static str,
        value: String,
    },

    /// The COLLADA document specified an unsupported version of the specification.
    ///
    /// The root `<COLLADA>` element of every COLLADA document must have a `version` attribute
    /// declaring which version of the specification the document conforms to. This library
    /// supports versions `1.4.0`, `1.4.1`, and `1.5.0`. If any other version is used, this error
    /// is returned.
    UnsupportedVersion {
        version: String,
    },

    /// There was an invalid URI fragment in the document.
    UriFragmentParseError(UriFragmentParseError),

    /// The XML in the document was malformed in some way.
    ///
    /// Not much more to say about this one ¯\_(ツ)_/¯
    XmlError(XmlError),
}

impl From<::chrono::format::ParseError> for ErrorKind {
    fn from(from: ::chrono::format::ParseError) -> ErrorKind {
        ErrorKind::TimeError(from)
    }
}

impl From<::std::num::ParseFloatError> for ErrorKind {
    fn from(from: ::std::num::ParseFloatError) -> ErrorKind {
        ErrorKind::ParseFloatError(from)
    }
}

impl From<::std::num::ParseIntError> for ErrorKind {
    fn from(from: ::std::num::ParseIntError) -> ErrorKind {
        ErrorKind::ParseIntError(from)
    }
}

impl From<::std::string::ParseError> for ErrorKind {
    fn from(from: ::std::string::ParseError) -> ErrorKind {
        match from {}
    }
}

impl From<UriFragmentParseError> for ErrorKind {
    fn from(from: UriFragmentParseError) -> ErrorKind {
        ErrorKind::UriFragmentParseError(from)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        match *self {
            ErrorKind::MissingAttribute { ref element, ref attribute } => {
                write!(formatter, "<{}> is missing the required attribute \"{}\"", element, attribute)
            }

            ErrorKind::MissingElement { ref expected, ref parent } => {
                if expected.len() == 1 {
                    write!(formatter, "<{}> is missing a required child element: {}", parent, expected[0])
                } else {
                    write!(formatter, "<{}> is missing a required child element (may be one of {}", parent, expected[0])?;
                    for element in &expected[1..] {
                        write!(formatter, ", {}", element)?;
                    }
                    write!(formatter, ")")
                }
            }

            ErrorKind::MissingValue { element } => {
                write!(formatter, "<{}> is missing required text data", element)
            }

            ErrorKind::ParseFloatError(ref error) => {
                error.fmt(formatter)
            }

            ErrorKind::ParseIntError(ref error) => {
                error.fmt(formatter)
            }

            ErrorKind::TimeError(ref error) => {
                error.fmt(formatter)
            }

            ErrorKind::UnexpectedAttribute { ref element, ref attribute, ref expected } => {
                write!(
                    formatter,
                    "<{}> had an an attribute \"{}\" that is not allowed, only the following attributes are allowed for <{0}>: {}",
                    element,
                    attribute,
                    StringListDisplay(&*expected),
                )
            }

            ErrorKind::UnexpectedCharacterData { ref element, data: _ } => {
                write!(formatter, "<{}> contained non-markup text data which isn't allowed", element)
            }

            ErrorKind::UnexpectedElement { ref parent, ref element, ref expected } => {
                write!(
                    formatter,
                    "<{}> had a child <{}> which is not allowed, <{0}> may only have the following children: {}",
                    parent,
                    element,
                    StringListDisplay(&*expected),
                )
            }

            ErrorKind::UnexpectedRootElement { ref element } => {
                write!(formatter, "Document began with <{}> instead of <COLLADA>", element)
            }

            ErrorKind::InvalidValue { ref element, ref value } => {
                write!(formatter, "<{}> contained an unexpected value {:?}", element, value)
            }

            ErrorKind::UnsupportedVersion { ref version } => {
                write!(formatter, "Unsupported COLLADA version {:?}, supported versions are \"1.4.0\", \"1.4.1\", \"1.5.0\"", version)
            }

            ErrorKind::UriFragmentParseError(ref error) => {
                error.fmt(formatter)
            }

            ErrorKind::XmlError(ref error) => {
                write!(formatter, "{}", error.msg())
            }
        }
    }
}

/// A specialized result type for COLLADA parsing.
///
/// Specializes [`std::result::Result`][std::result::Result] to [`Error`][Error] for the purpose
/// of simplifying the signature of any falible COLLADA operation.
///
/// [std::result::Result]: https://doc.rust-lang.org/std/result/enum.Result.html
/// [Error]: struct.Error.html
pub type Result<T> = std::result::Result<T, Error>;
