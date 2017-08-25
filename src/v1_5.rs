//! Type definitions matching the COLLADA `1.5.0` specification.

use {Result, Error, ErrorKind};
use common::*;
use std::io::Read;
use utils;
use utils::*;
use xml::common::Position;
use xml::reader::EventReader;

/// Represents a parsed COLLADA document.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "COLLADA"]
pub struct Collada {
    /// The version string for the COLLADA specification used by the document.
    ///
    /// Only "1.4.0", "1.4.1", and "1.5.0" are supported currently.
    #[attribute]
    pub version: String,

    // Included for completeness in parsing, not actually used.
    #[attribute]
    pub xmlns: Option<String>,

    /// The base uri for any relative URIs in the document.
    ///
    /// Specified by the `base` attribute on the root `<COLLADA>` element.
    #[attribute]
    #[name = "base"]
    pub base_uri: Option<AnyUri>,

    /// Global metadata about the COLLADA document.
    #[child]
    pub asset: Asset,

    #[child]
    pub libraries: Vec<Library>,

    #[child]
    pub scene: Option<Scene>,

    #[child]
    pub extras: Vec<Extra>,
}

impl Collada {
    /// Read a COLLADA document from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use collaborate::v1_5::Collada;
    ///
    /// static DOCUMENT: &'static str = r#"
    ///     <?xml version="1.0" encoding="utf-8"?>
    ///     <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
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
        Self::parse(reader)
    }

    /// Attempts to parse the contents of a COLLADA document.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use std::fs::File;
    /// use collaborate::v1_5::Collada;
    ///
    /// let file = File::open("resources/v1_5_minimal.dae").unwrap();
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
        Self::parse(reader)
    }

    pub fn parse<R: Read>(mut reader: EventReader<R>) -> Result<Collada> {
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

        if version != "1.5.0" {
            return Err(Error {
                position: reader.position(),
                kind: ErrorKind::UnsupportedVersion {
                    version: version,
                },
            });
        }

        Collada::parse_element(&mut reader, element_start)
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

/// Provides arbitrary additional information about an element.
///
/// COLLADA allows for applications to provide extra information about any given piece of data,
/// including application-specific information that's not part of the COLLADA specification. This
/// data can be any syntactically valid XML data, and is not parsed as part of this library, save
/// for a few specific 3rd party applications that are directly supported.
///
/// # Choosing a Technique
///
/// There may be more than one [`Technique`][Technique] provided in `techniques`, but generally
/// only one is used by the consuming application. The application should pick a technique
/// with a supported profile. If there are multiple techniques with supported profiles the
/// application is free to pick whichever technique is preferred.
///
/// [Technique]: struct.Technique.html
#[derive(Debug, Clone, Default, PartialEq, ColladaElement)]
#[name = "extra"]
pub struct Extra {
    /// The identifier of the element, if present. Will be unique within the document.
    #[attribute]
    pub id: Option<String>,

    /// The text string name of the element, if present.
    #[attribute]
    pub name: Option<String>,

    /// A hint as to the type of information this element represents, if present. Must be
    /// must be understood by the consuming application.
    #[attribute]
    #[name = "type"]
    pub type_hint: Option<String>,

    /// Asset-management information for this element, if present.
    ///
    /// While this is technically allowed in all `<extra>` elements, it is likely only present in
    /// elements that describe a new "asset" of some kind, rather than in `<extra>` elements that
    /// provide application-specific information about an existing one.
    #[child]
    pub asset: Option<Asset>,

    /// The arbitrary additional information, containing unprocessed XML events. There will always
    /// be at least one item in `techniques`.
    #[child]
    #[required]
    pub techniques: Vec<Technique>,
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

#[derive(Debug, Clone, PartialEq, ColladaElement)]
pub enum Library {
    Animations(LibraryAnimations),
    AnimationClips(LibraryAnimationClips),
    ArticulatedSystmes(LibraryArticulatedSystems),
    Cameras(LibraryCameras),
    Controllers(LibraryControllers),
    Effects(LibraryEffects),
    ForceFields(LibraryForceFields),
    Formulas(LibraryFormulas),
    Geometries(LibraryGeometries),
    Images(LibraryImages),
    Joints(LibraryJoints),
    KinematicsModels(LibraryKinematicsModels),
    KinematicsScenes(LibraryKinematicsScenes),
    Lights(LibraryLights),
    Materials(LibraryMaterials),
    Nodes(LibraryNodes),
    PhysicsMaterials(LibraryPhysicsMaterials),
    PhysicsModels(LibraryPhysicsModels),
    PhysicsScenes(LibraryPhysicsScenes),
    VisualScenes(LibraryVisualScenes),
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_animations"]
pub struct LibraryAnimations;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_animation_clips"]
pub struct LibraryAnimationClips;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_articulated_systems"]
pub struct LibraryArticulatedSystems;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_cameras"]
pub struct LibraryCameras;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_controllers"]
pub struct LibraryControllers;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_effects"]
pub struct LibraryEffects;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_force_fields"]
pub struct LibraryForceFields;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_formulas"]
pub struct LibraryFormulas;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_geometries"]
pub struct LibraryGeometries;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_images"]
pub struct LibraryImages;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_joints"]
pub struct LibraryJoints;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_kinematics_models"]
pub struct LibraryKinematicsModels;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_kinematics_scenes"]
pub struct LibraryKinematicsScenes;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_lights"]
pub struct LibraryLights;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_materials"]
pub struct LibraryMaterials;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_nodes"]
pub struct LibraryNodes;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_physics_materials"]
pub struct LibraryPhysicsMaterials;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_physics_models"]
pub struct LibraryPhysicsModels;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_physics_scenes"]
pub struct LibraryPhysicsScenes;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_visual_scenes"]
pub struct LibraryVisualScenes;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "scene"]
pub struct Scene;

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
