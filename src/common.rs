use {AnyUri, Error, ErrorKind, Result};
use std::io::Read;
use utils::*;
use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};

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
