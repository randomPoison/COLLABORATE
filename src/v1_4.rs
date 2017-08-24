use {AnyUri, DateTime, Error, ErrorKind, Result, Unit, UpAxis, utils};
use common::*;
use std::io::Read;
use utils::*;
use xml::common::Position;
use xml::reader::EventReader;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "COLLADA"]
pub struct Collada {
    #[attribute]
    pub version: String,

    #[attribute]
    pub xmlns: Option<String>,

    #[attribute]
    pub base: Option<AnyUri>,

    #[child]
    pub asset: Asset,

    #[child]
    pub libraries: Vec<LibraryElement>,

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
        Self::parse(reader)
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

        if version != "1.4.0" && version != "1.4.1" {
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

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "asset"]
pub struct Asset {
    #[child]
    pub contributors: Vec<Contributor>,

    #[child]
    pub created: DateTime,

    #[child]
    pub keywords: Option<String>,

    #[child]
    pub modified: DateTime,

    #[child]
    pub revision: Option<String>,

    #[child]
    pub subject: Option<String>,

    #[child]
    pub title: Option<String>,

    #[child]
    #[optional_with_default]
    pub unit: Unit,

    #[child]
    #[optional_with_default]
    pub up_axis: UpAxis,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ColladaElement)]
#[name = "contributor"]
pub struct Contributor {
    #[child]
    pub author: Option<String>,

    #[child]
    pub authoring_tool: Option<String>,

    #[child]
    pub comments: Option<String>,

    #[child]
    pub copyright: Option<String>,

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

#[derive(Debug, Clone, PartialEq, ColladaElement)]
pub enum LibraryElement {
    Animations(LibraryAnimations),
    AnimationClips(LibraryAnimationClips),
    Cameras(LibraryCameras),
    Controllers(LibraryControllers),
    Effects(LibraryEffects),
    ForceFields(LibraryForceFields),
    Geometries(LibraryGeometries),
    Images(LibraryImages),
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
#[name = "library_geometries"]
pub struct LibraryGeometries {
    #[attribute]
    pub id: Option<String>,

    #[attribute]
    pub name: Option<String>,

    #[child]
    pub asset: Option<Asset>,

    #[child]
    #[required]
    pub geometry: Vec<Geometry>,

    #[child]
    pub extra: Vec<Extra>,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_images"]
pub struct LibraryImages;

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
#[name = "geometry"]
pub struct Geometry {
    #[attribute]
    pub id: String,

    #[attribute]
    pub name: String,

    #[child]
    pub asset: Option<Asset>,

    #[child]
    pub geometric_element: GeometricElement,

    #[child]
    pub extra: Vec<Extra>,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
pub enum GeometricElement {
    ConvexMesh(ConvexMesh),
    Mesh(Mesh),
    Spline(Spline),
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "convex_mesh"]
pub struct ConvexMesh;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "mesh"]
pub struct Mesh;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "scene"]
pub struct Scene;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "spline"]
pub struct Spline;
