//! Type definitions common to all supported COLLADA specifications.

use {Error, ErrorKind, Result};
use std::io::Read;
use std::str::FromStr;
use utils;
use utils::*;
use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};

/// A URI in the COLLADA document.
///
/// Represents the [`xs:anyURI`][anyURI] XML data type.
///
/// [anyURI]: http://www.datypic.com/sc/xsd/t-xsd_anyURI.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyUri(String);

impl From<String> for AnyUri {
    fn from(from: String) -> AnyUri {
        AnyUri(from)
    }
}

// TODO: Actually parse the string and verify that it's a valid URI.
impl ::std::str::FromStr for AnyUri {
    type Err = ::std::string::ParseError;

    fn from_str(string: &str) -> ::std::result::Result<AnyUri, ::std::string::ParseError> {
        Ok(AnyUri(string.into()))
    }
}

/// A datetime value, with or without a timezone.
///
/// Timestamps in a COLLADA document adhere to [ISO 8601][ISO 8601], which specifies a standard
/// format for writing a date and time value, with or without a timezone. Since the timezone
/// component is optional, the `DateTime` object will preserve the timezone if one was specified,
/// or it will be considered a "naive" datetime if it does not.
///
/// The [`chrono`][chrono] crate is used for handling datetime types, and its API is re-exported
/// for convenience.
///
/// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
/// [chrono]: https://docs.rs/chrono
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTime {
    /// A timestamp with a known timezone, specified as a fixed offset from UTC.
    Utc(::chrono::DateTime<::chrono::FixedOffset>),

    /// A timestamp with no timezone.
    Naive(::chrono::NaiveDateTime),
}

impl FromStr for DateTime {
    type Err = ::chrono::ParseError;

    fn from_str(source: &str) -> ::std::result::Result<DateTime, ::chrono::ParseError> {
        source
            .parse()
            .map(|datetime| DateTime::Utc(datetime))
            .or_else(|_| {
                ::chrono::NaiveDateTime::from_str(source)
                    .map(DateTime::Naive)
            })
    }
}

/// Arbitrary additional information represented as XML events.
///
/// > TODO: Provide more information about processing techniques.
#[derive(Debug, Clone, PartialEq)]
pub struct Technique {
    /// A vendor-defined string that indicates the platform or capability target for the technique.
    /// Consuming applications need not support all (or any) profiles, and can safely ignore
    /// techniques with unknown or unsupported profiles.
    pub profile: String,

    /// The schema used for validating the contents of the `<technique>` element.
    ///
    /// Currently, validation is not performed by this library, and is left up to the consuming
    /// application.
    pub xmlns: Option<AnyUri>,

    /// The raw XML events for the data contained within the technique. These events do not contain
    /// the `StartElement` and `EndElement` events for the `<technique>` element itself. As such,
    /// the contents of `data` do not represent a valid XML document, as they may not have a single
    /// root element.
    pub data: Vec<XmlEvent>,
}

impl ColladaElement for Technique {
    fn name_test(name: &str) -> bool {
        name == "technique"
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<Technique>
    where
        R: Read,
    {
        let mut profile = None;
        let mut xmlns = None;
        let mut data = Vec::default();

        for attribute in element_start.attributes {
            match &*attribute.name.local_name {
                "profile" => { profile = Some(attribute.value); }

                "xmlns" => { xmlns = Some(attribute.value.into()); }

                _ => {
                    return Err(Error {
                        position: reader.position(),
                        kind: ErrorKind::UnexpectedAttribute {
                            element: "technique",
                            attribute: attribute.name.local_name.clone(),
                            expected: vec!["profile", "xmlns"],
                        },
                    });
                }
            }
        }

        let profile = match profile {
            Some(profile) => { profile }

            None => {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::MissingAttribute {
                        element: "technique",
                        attribute: "profile",
                    },
                });
            }
        };

        let mut depth = 0;
        loop {
            let event = reader.next()?;
            match event {
                XmlEvent::StartElement { ref name, .. } if name.local_name == "technique" => { depth += 1; }

                XmlEvent::EndElement { ref name } if name.local_name == "technique" => {
                    if depth == 0 {
                        break;
                    } else {
                        depth -= 1;
                    }
                }

                _ => {}
            }

            data.push(event);
        }

        Ok(Technique {
            profile: profile,
            xmlns: xmlns,
            data: data,
        })
    }

    fn add_names(names: &mut Vec<&'static str>) {
        names.push("technique");
    }
}

/// Defines the unit of distance for an [`Asset`][Asset].
///
/// The unit of distance applies to all spatial measurements for the [`Asset`][Asset], unless
/// overridden by a more local `Unit`. A `Unit` is self-describing, providing both its name and
/// length in meters, and does not need to be consistent with any real-world measurement.
///
/// [Asset]: struct.Asset.html
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "unit"]
pub struct Unit {
    /// The name of the distance unit. For example, “meter”, “centimeter”, “inch”, or “parsec”.
    /// This can be the name of a real measurement, or an imaginary name. Defaults to `1.0`.
    #[attribute]
    pub meter: f64,

    /// How many real-world meters in one distance unit as a floating-point number. For example,
    /// 1.0 for the name "meter"; 1000 for the name "kilometer"; 0.3048 for the name
    /// "foot". Defaults to "meter".
    #[attribute]
    pub name: String,
}

impl Default for Unit {
    fn default() -> Unit {
        Unit {
            meter: 1.0,
            name: "meter".into(),
        }
    }
}

/// Describes the coordinate system for an [`Asset`][Asset].
///
/// All coordinates in a COLLADA document are right-handed, so describing the up axis alone is
/// enough to determine the other two axis. The table below shows all three possibilites:
///
/// | Value       | Right Axis | Up Axis    | In Axis    |
/// |-------------|------------|------------|------------|
/// | `UpAxis::X` | Negative Y | Positive X | Positive Z |
/// | `UpAxis::Y` | Positive X | Positive Y | Positive Z |
/// | `UpAxis::Z` | Positive X | Positive Z | Negative Y |
///
/// [Asset]: struct.Asset.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpAxis {
    X,
    Y,
    Z,
}

impl ColladaElement for UpAxis {
    fn name_test(name: &str) -> bool {
        name == "up_axis"
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<UpAxis>
    where
        R: Read,
    {
        utils::verify_attributes(reader, "up_axis", element_start.attributes)?;
        let text: String = utils::optional_text_contents(reader, "up_axis")?.unwrap_or_default();
        let parsed = match &*text {
            "X_UP" => { UpAxis::X }
            "Y_UP" => { UpAxis::Y }
            "Z_UP" => { UpAxis::Z }
            _ => {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::InvalidValue {
                        element: "up_axis".into(),
                        value: text,
                    },
                });
            }
        };

        Ok(parsed)
    }

    fn add_names(names: &mut Vec<&'static str>) {
        names.push("up_axis");
    }
}

impl Default for UpAxis {
    fn default() -> UpAxis { UpAxis::Y }
}
