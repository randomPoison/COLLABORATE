use {Result, Error, ErrorKind};
use self::ChildOccurrences::*;
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::str::FromStr;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::{EventReader, ParserConfig};
use xml::reader::XmlEvent::*;

pub static PARSER_CONFIG: ParserConfig = ParserConfig {
    trim_whitespace: true,
    whitespace_to_characters: true,
    cdata_to_characters: true,
    ignore_comments: true,
    coalesce_characters: true,
};

/// Helper trait for handling parsing. This can be derived for most types with the
/// `collaborate-derive` crate.
pub trait ColladaElement: Sized {
    /// Tests whether `name` is a valid name for the element or group.
    ///
    /// This allows multiple elements to be grouped together in a single enum type.
    fn name_test(name: &str) -> bool;

    /// Parses the current element from the event stream.
    ///
    /// Implementation should panic if `element_start` isn't valid for the current element.
    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<Self>
    where
        R: Read;

    /// Adds all valid names for the current element to `names`.
    ///
    /// This allows both single elements and element groups to add their name(s) to the list of
    /// expected names when returning an error message.
    fn add_names(names: &mut Vec<&'static str>);
}

#[derive(Debug)]
pub struct ElementStart {
    pub name: OwnedName,
    pub attributes: Vec<OwnedAttribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildOccurrences {
    Optional,
    OptionalWithDefault,
    Required,
    Many,
    RequiredMany,
}

pub struct ElementConfiguration<'a, R: 'a + Read> {
    pub name: &'static str,
    pub children: &'a mut [ChildConfiguration<'a, R>],
    pub text_contents: Option<&'a mut FnMut(&mut EventReader<R>, String) -> Result<()>>,
}

impl<'a, R: 'a + Read> ElementConfiguration<'a, R> {
    pub fn parse_children(self, reader: &mut EventReader<R>) -> Result<()> {
        // Keep track of the text position for the root element so that it can be used for error
        // messages.
        let root_position = reader.position();

        if let Some(handle_text) = self.text_contents {
            let contents = required_text_contents(reader, self.name)?;
            handle_text(reader, contents)?;
            return Ok(());
        }

        // The index of the next child we are expecting.
        let mut current_child = 0;

        // Whether or not we have encountered the current child at least once. This is only used
        // for `RequiredMany` children to ensure they are found at least once.
        let mut has_encountered_child = false;

        'elements: while let Some(element) = start_element(reader, self.name)? {
            while current_child < self.children.len() {
                let child = &mut self.children[current_child];

                if (child.name)(&*element.name.local_name) {
                    has_encountered_child = true;

                    // We've found a valid child, hooray! Allow it to run its parsing code.
                    (child.action)(reader, element)?;

                    // Either advance `current_child` or don't, depending on if it's allowed to repeat.
                    match child.occurrences {
                        Optional | OptionalWithDefault | Required => {
                            // Advance current child.
                            has_encountered_child = false;
                            current_child += 1;
                        }
                        Many | RequiredMany => { /* Don't advance current child. */ }
                    }

                    continue 'elements;
                }

                // The element didn't match the current child. Check to see if the current child
                // is required. If so, we return an error if we never encountered it.
                if child.occurrences == Required || (child.occurrences == RequiredMany && !has_encountered_child) {
                    break;
                }

                // Advance the current child.
                has_encountered_child = false;
                current_child += 1;
            }

            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    parent: self.name,
                    element: element.name.local_name,
                    expected: self.collect_expected_children(),
                },
            });
        }

        // No more child elements are present, and none of the children we encountered were invalid.
        // Verify that there are no remaining required children.
        for child in &self.children[current_child..] {
            if child.occurrences == ChildOccurrences::Required {
                let mut expected = Vec::new();
                (child.add_names)(&mut expected);

                return Err(Error {
                    position: root_position,
                    kind: ErrorKind::MissingElement {
                        parent: self.name,
                        expected: expected,
                    },
                });
            }
        }

        Ok(())
    }

    fn collect_expected_children(&self) -> Vec<&'static str> {
        let mut names = Vec::with_capacity(self.children.len());
        for child in self.children.iter() {
            (child.add_names)(&mut names);
        }
        names
    }
}

pub struct ChildConfiguration<'a, R: 'a + Read> {
    pub name: &'a Fn(&str) -> bool,
    pub occurrences: ChildOccurrences,
    pub action: &'a mut FnMut(&mut EventReader<R>, ElementStart) -> Result<()>,
    pub add_names: &'a Fn(&mut Vec<&'static str>),
}

pub fn get_document_start<R: Read>(reader: &mut EventReader<R>) -> Result<ElementStart> {
    // Eat the `StartDocument` event. It has no useful information for our purposes, but it
    // will always be the first event emitted, even if there's no XML declaration at the
    // beginning of the document. This is defined as part of the xml-rs API as of v0.3.5,
    // but it's possible this can will change in the future.
    match reader.next()? {
        StartDocument { .. } => {},
        _ => panic!("First event from EventReader wasn't StartDocument"),
    }

    // The next element will always be the `<COLLADA>` tag. This will specify what version of
    // the COLLADA spec is being used, which is how we'll determine our sub-parser.
    let element_start = match reader.next()? {
        StartElement { name, attributes, namespace: _ } => {
            // If the element isn't the `<COLLADA>` tag then the document is malformed,
            // return an error.
            if name.local_name != "COLLADA" {
                return Err(Error {
                    position: reader.position(),
                    kind: ErrorKind::UnexpectedRootElement {
                        element: name.local_name,
                    }
                })
            }

            ElementStart { name, attributes }
        }

        // I'm *almost* 100% certain that the only event that can follow the `StartDocument`
        // event is a `StartElement` event. As of v0.3.5, xml-rs doesn't support
        // `<!DOCTYPE>` or processing instructions, and it ignores whitespace and comments
        // (according to how we configure the parser), and those are the only things allowed
        // between `StartDocument` and the first `StartElement`. If xml-rs changes its
        // behavior this will need to be updated.
        event @ _ => { panic!("Unexpected event: {:?}", event); }
    };

    Ok(element_start)
}

pub fn start_element<R: Read>(
    reader: &mut EventReader<R>,
    parent: &'static str,
) -> Result<Option<ElementStart>> {
    match reader.next()? {
        StartElement { name, attributes, namespace: _ } => {
            return Ok(Some(ElementStart { name, attributes }));
        }

        EndElement { name } => {
            debug_assert_eq!(parent, name.local_name);
            return Ok(None);
        }

        Characters(data) => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedCharacterData {
                    element: parent,
                    data: data,
                }
            })
        }

        // TODO: How do we handle processing instructions? I suspect we want to just skip them, but
        // I'm not sure.
        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

pub fn required_text_contents<R, T>(
    reader: &mut EventReader<R>,
    parent: &'static str,
) -> Result<T>
    where
    R: Read,
    T: FromStr,
    ErrorKind: From<<T as FromStr>::Err>,
{
    match reader.next()? {
        Characters(data) => {
            let result = T::from_str(&*data)
                .map_err(|error| Error {
                    position: reader.position(),
                    kind: error.into(),
                })?;
            end_element(reader, parent)?;
            return Ok(result);
        }

        StartElement { name, attributes: _, namespace: _ } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    parent: parent,
                    element: name.local_name,
                    expected: vec![],
                },
            })
        }

        EndElement { .. } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::MissingValue {
                    element: parent,
                },
            });
        }

        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

pub fn optional_text_contents<R, T>(
    reader: &mut EventReader<R>,
    parent: &'static str,
) -> Result<Option<T>>
    where
    R: Read,
    T: FromStr,
    ErrorKind: From<<T as FromStr>::Err>
{
    match reader.next()? {
        Characters(data) => {
            let result = T::from_str(&*data)
                .map_err(|error| Error {
                    position: reader.position(),
                    kind: error.into(),
                })?;
            end_element(reader, parent)?;
            return Ok(Some(result));
        }

        StartElement { name, attributes: _, namespace: _ } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    parent: parent,
                    element: name.local_name,
                    expected: vec![],
                },
            })
        }

        EndElement { .. } => {
            return Ok(None);
        }

        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

pub fn end_element<R: Read>(reader: &mut EventReader<R>, parent: &'static str) -> Result<()> {
    match reader.next()? {
        EndElement { .. } => {
            return Ok(());
        }

        StartElement { name, attributes: _, namespace: _ } => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedElement {
                    parent: parent,
                    element: name.local_name,
                    expected: vec![],
                },
            })
        }

        Characters(data) => {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnexpectedCharacterData {
                    element: parent.into(),
                    data: data,
                }
            })
        }

        ProcessingInstruction { .. } => { unimplemented!(); }

        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }
}

/// Meaning, of course, "verify that there are no attributes".
pub fn verify_attributes<R: Read>(reader: &EventReader<R>, name: &'static str, attributes: Vec<OwnedAttribute>) -> Result<()> {
    // Make sure the child element has no attributes.
    if attributes.len() != 0 {
        return Err(Error {
            position: reader.position(),
            kind: ErrorKind::UnexpectedAttribute {
                element: name,
                attribute: attributes[0].name.local_name.clone(),
                expected: vec![],
            },
        })
    }

    Ok(())
}

// TODO: This is a temporary helper to allow us to ignore COLLADA elements that we don't care
// about parsing yet. This should be removed once we've implemented the full COLLADA spec.
pub fn stub_out<R>(reader: &mut EventReader<R>, stubbed_name: &str) -> Result<()> where R: Read {
    let mut depth = 1;
    loop {
        match reader.next()? {
            StartElement { name, attributes: _, namespace: _ } => {
                if name.local_name == stubbed_name { depth += 1; }
            }

            EndElement { name } => {
                if name.local_name == stubbed_name { depth -= 1; }
                if depth == 0 { break; }
            }

            _ => {}
        }
    }

    Ok(())
}

/// Helper struct for pretty-printing lists of strings.
pub struct StringListDisplay<'a>(pub &'a [&'a str]);

impl<'a> Display for StringListDisplay<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> ::std::result::Result<(), fmt::Error> {
        if self.0.len() > 0 {
            write!(formatter, "{}", self.0[0])?;

            for string in &self.0[1..] {
                write!(formatter, ", {}", string)?;
            }
        }

        Ok(())
    }
}
