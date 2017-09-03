//! Type definitions matching the COLLADA `1.4.1` specification.
//!
//! Note that the COLLADA `1.4.0` specification is subsumed by the `1.4.1` spec, so `1.4.0`
//! documents are still accurately represented by the types in this module. Users of COLLABORATE
//! don't need to distinguish between `1.4.0` and `1.4.1` documents.

use {Error, ErrorKind, Result};
use common::*;
use std::io::Read;
use utils;
use utils::*;
use xml::common::Position;
use xml::reader::EventReader;

/// Represents a complete COLLADA document.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "COLLADA"]
pub struct Collada {
    /// The version string for the COLLADA specification used by the document.
    ///
    /// Will be "1.4.0" or "1.4.1".
    #[attribute]
    pub version: String,

    /// Included for completeness in parsing, not actually used.
    // TODO: Can we remove `xmlns`? Should we remove it?
    #[attribute]
    pub xmlns: Option<String>,

    /// The base uri for any relative URIs in the document.
    ///
    /// Refer to the [XML Base Specification](https://www.w3.org/TR/xmlbase/).
    #[attribute]
    #[name = "base"]
    pub base_uri: Option<AnyUri>,

    /// Global metadata about the COLLADA document.
    #[child]
    pub asset: Asset,

    /// The collection of libraries that bulk of the actual data contained in the document.
    ///
    /// Libraries can occur in any order, and there can be 0 or more libraries of any given type.
    /// Helper methods are provided to iterate over all instances of a given library type, as well
    /// as to extract data from all instance of a library type.
    // TODO: Actually provide the helper methods.
    #[child]
    pub libraries: Vec<Library>,

    /// Defines the scene hierarchy associated with this document.
    #[child]
    pub scene: Option<Scene>,

    /// Arbitrary additional information about the document as a whole.
    ///
    /// For more information about 3rd-party extensions, see the
    /// [crate-level documentation](../index.html#3rd-party-extensions).
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
    /// use collaborate::v1_4::Collada;
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
    /// a document is parsed see the [crate-level documentation](../index.html)
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
    /// use collaborate::v1_4::Collada;
    ///
    /// let file = File::open("resources/blender_cube.dae").unwrap();
    /// let collada = Collada::read(file).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation](../index.html).
    pub fn read<R: Read>(reader: R) -> Result<Collada> {
        let reader = EventReader::new_with_config(reader, utils::PARSER_CONFIG.clone());
        Self::parse(reader)
    }

    /// Helper method that handles the bulk of the parsing work.
    ///
    /// `from_str` and `read` just create the `EventReader<R>` instance and then defer to `parse`.
    fn parse<R: Read>(mut reader: EventReader<R>) -> Result<Collada> {
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

/// Describes a stream of values from an array data source.
///
/// An accessor declares an access pattern into an array of source data. The arrays can be
/// arranged in either an interleaved or noninterleaved manner, depending on the `offset` and
/// `stride` values.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "accessor"]
pub struct Accessor {
    /// The number of times the array is accessed.
    #[attribute]
    pub count: usize,

    /// The index of the first value to be read from the array.
    #[attribute]
    #[optional_with_default = "0"]
    pub offset: usize,

    /// The location of the array to access.
    ///
    /// This may refer to a COLLADA array element or to an array data source outside the scope
    /// of the instance document; The source does not need to be a COLLADA document.
    #[attribute]
    pub source: AnyUri,

    /// The number of values that are to be considered a unit during each access to the array.
    #[attribute]
    #[optional_with_default = "1"]
    pub stride: usize,

    #[child]
    pub params: Vec<Param>,
}

impl Accessor {
    /// Access a source array using the accessor.
    ///
    /// Returns a sub-slice of `array` containing the
    pub fn access<'a, 'b, T>(&'a self, array: &'b [T], index: usize) -> &'b [T] {
        let start = self.offset + self.stride * index;
        let end = start + self.stride;
        &array[start..end]
    }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
pub enum Array {
    Idref(IdrefArray),
    Name(NameArray),
    Bool(BoolArray),
    Float(FloatArray),
    Int(IntArray),
}

impl Array {
    pub fn as_float_array(&self) -> Option<&FloatArray> {
        match *self {
            Array::Float(ref float_array) => Some(float_array),
            _ => None,
        }
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

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "bool_array"]
pub struct BoolArray;

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
    pub source_data: Option<AnyUri>,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "convex_mesh"]
pub struct ConvexMesh;

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
#[name = "float_array"]
pub struct FloatArray {
    #[attribute]
    pub count: usize,

    #[attribute]
    pub id: Option<String>,

    #[attribute]
    pub name: Option<String>,

    #[attribute]
    #[optional_with_default = "6"]
    pub digits: usize,

    #[attribute]
    #[optional_with_default = "38"]
    pub magnitude: usize,

    #[text]
    pub data: Vec<f32>,
}

/// A geometric element of unknown type.
///
/// Each variant wraps a single value containing a given type of geometric data. See the
/// documentation for each of the possible geometric types for more information.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
pub enum GeometricElement {
    ConvexMesh(ConvexMesh),
    Mesh(Mesh),
    Spline(Spline),
}

impl GeometricElement {
    /// Attempts to downcast the geometric element to a [`ConvexMesh`].
    ///
    /// Returns a reference to the inner [`ConvexMesh`] if there is one, returns `None` otherwise.
    /// This is useful if you have a `GeometricElement` but only want to use it if it represents a
    /// [`ConvexMesh`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ::collaborate::v1_4::*;
    /// # static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");
    /// # let source = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    /// # let document = Collada::from_str(&*source).unwrap();
    /// # let library_geometries = document.libraries[5].as_library_geometries().unwrap();
    /// let geometry = &library_geometries.geometries[0];
    /// if let Some(mesh) = geometry.geometric_element.as_convex_mesh() {
    ///     // Do something with `mesh`.
    /// }
    /// ```
    ///
    /// [`ConvexMesh`]: ./struct.ConvexMesh.html
    pub fn as_convex_mesh(&self) -> Option<&ConvexMesh> {
        match *self {
            GeometricElement::ConvexMesh(ref mesh) => Some(mesh),
            _ => None,
        }
    }

    /// Attempts to downcast the geometric element to a [`Mesh`].
    ///
    /// Returns a reference to the inner [`Mesh`] if there is one, returns `None` otherwise. This
    /// is useful if you have a `GeometricElement` but only want to use it if it represents a
    /// [`Mesh`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ::collaborate::v1_4::*;
    /// # static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");
    /// # let source = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    /// # let document = Collada::from_str(&*source).unwrap();
    /// # let library_geometries = document.libraries[5].as_library_geometries().unwrap();
    /// let geometry = &library_geometries.geometries[0];
    /// if let Some(mesh) = geometry.geometric_element.as_mesh() {
    ///     // Do something with `mesh`.
    /// }
    /// ```
    ///
    /// [`Mesh`]: ./struct.Mesh.html
    pub fn as_mesh(&self) -> Option<&Mesh> {
        match *self {
            GeometricElement::Mesh(ref mesh) => Some(mesh),
            _ => None,
        }
    }

    /// Attempts to downcast the geometric element to a [`Spline`].
    ///
    /// Returns a reference to the inner [`Spline`] if there is one, returns `None` otherwise. This
    /// is useful if you have a `GeometricElement` but only want to use it if it represents a
    /// [`Spline`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ::collaborate::v1_4::*;
    /// # static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");
    /// # let source = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    /// # let document = Collada::from_str(&*source).unwrap();
    /// # let library_geometries = document.libraries[5].as_library_geometries().unwrap();
    /// let geometry = &library_geometries.geometries[0];
    /// if let Some(spline) = geometry.geometric_element.as_spline() {
    ///     // Do something with `spline`.
    /// }
    /// ```
    ///
    /// [`Spline`]: ./struct.Spline.html
    pub fn as_spline(&self) -> Option<&Spline> {
        match *self {
            GeometricElement::Spline(ref mesh) => Some(mesh),
            _ => None,
        }
    }
}

/// Describes the visual shape and appearance of an object in a scene.
///
/// The primary purpose of `Geometry` is to provide access to a [`GeometricElement`], via its
/// `geographic_element` member. It contains miscellaneous additional data, such as asset
/// metadata, but otherwise does not directly contain any geometric data.
///
/// # Examples
///
/// ```
/// # use ::collaborate::v1_4::*;
/// # static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");
/// # let source = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
/// # let document = Collada::from_str(&*source).unwrap();
/// # let library_geometries = document.libraries[5].as_library_geometries().unwrap();
/// let geometry = &library_geometries.geometries[0];
/// match geometry.geometric_element {
///     GeometricElement::ConvexMesh(ref mesh) => {
///         // Do something with `mesh`.
///     }
///
///     GeometricElement::Mesh(ref mesh) => {
///         // Do something with `mesh`.
///     }
///
///     GeometricElement::Spline(ref spline) => {
///         // Do something with `spline`.
///     }
/// }
/// ```
///
/// [`GeometricElement`]: ./enum.GeometricElement.html
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "geometry"]
pub struct Geometry {
    /// A unique identifier for the geometry instance.
    ///
    /// Will be unique within the document.
    #[attribute]
    pub id: Option<String>,

    /// The human-friendly name for this geometry instance.
    ///
    /// Has no semantic meaning.
    #[attribute]
    pub name: Option<String>,

    /// Metadata about this geometry instance and the data it contains.
    #[child]
    pub asset: Option<Asset>,

    /// The actual data for the geometry instance.
    #[child]
    pub geometric_element: GeometricElement,

    /// Arbitrary additional information about this geometry instance and the data it contains.
    ///
    /// For more information about 3rd-party extensions, see the
    /// [crate-level documentation](../index.html#3rd-party-extensions).
    #[child]
    pub extra: Vec<Extra>,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "IDREF_array"]
pub struct IdrefArray;

#[derive(Debug, Clone)]
pub struct InputsForOffset<'a> {
    inputs: ::std::slice::Iter<'a, SharedInput>,
    offset: usize,
}

impl<'a> Iterator for InputsForOffset<'a> {
    type Item = &'a SharedInput;

    fn next(&mut self) -> Option<&'a SharedInput> {
        while let Some(input) = self.inputs.next() {
            if input.offset == self.offset {
                return Some(input);
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "int_array"]
pub struct IntArray;

/// A single library of unknown type.
///
/// Each variant wraps a single value containing the library data. See the documentation for
/// each of the possible library types for more information on what data each can contain.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
pub enum Library {
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

impl Library {
    pub fn as_library_geometries(&self) -> Option<&LibraryGeometries> {
        match *self {
            Library::Geometries(ref library_geometries) => Some(library_geometries),
            _ => None,
        }
    }
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

/// Contains geometric data for the document.
///
/// The geometric data is contained in `geometries` by one or more [`Geometry`] instances,
/// `LibraryGeometries` is only a container and does not represent any geometric data itself.
///
/// [`Geometry`]: ./struct.Geometry.html
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "library_geometries"]
pub struct LibraryGeometries {
    /// A unique identifier for the library.
    ///
    /// Will be unique within the document.
    #[attribute]
    pub id: Option<String>,

    /// The human-friendly name for this library.
    ///
    /// Has no semantic meaning.
    #[attribute]
    pub name: Option<String>,

    /// Metada about the library and the data contained within it.
    #[child]
    pub asset: Option<Asset>,

    /// The geometric data contained within this library instance.
    ///
    /// There will always be at least one geometric element in a `LibraryGeometries`.
    #[child]
    #[required]
    pub geometries: Vec<Geometry>,

    /// Arbitrary additional information about this library and the data it contains.
    ///
    /// For more information about 3rd-party extensions, see the
    /// [crate-level documentation](../index.html#3rd-party-extensions).
    #[child]
    pub extras: Vec<Extra>,
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
#[name = "lines"]
pub struct Lines;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "linestrips"]
pub struct Linestrips;

/// Describes basic geometric meshes using vertex and primitive information.
///
/// Meshes embody a general form of geometric description that primarily includes vertex and
/// primitive information. Vertex information is the set of attributes associated with a poin on
/// the surface of the mesh. Each vertex includes data for attributes such as:
///
/// * Vertex position
/// * Vertex color
/// * Vertex normal
/// * Vertex texture coordinate
///
/// The mesh also includes a description of how the vertices are organized to form the geometric
/// shape of the mesh. The mesh vertices are collated into geometric primitives such as polygons,
/// triangles, or lines.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "mesh"]
pub struct Mesh {
    /// One or more [`Source`] instances containing the raw mesh data.
    ///
    /// These contain the raw data used to specify the vertex attributes of the vertices in the
    /// mesh. The primitives in `primitives` will index into these sources to specify the mesh.
    ///
    /// [`Source`]: ./struct.Source.html
    #[child]
    #[required]
    pub sources: Vec<Source>,

    /// Describes the mesh's vertex attributes.
    ///
    /// `vertices` will have the [`UnsharedInput`] which specifies the "POSITION" attribute for
    /// the mesh's vertices. It may also specify other mesh attributes.
    ///
    /// [`UnsharedInput`]: ./struct.UnsharedInput.html
    #[child]
    pub vertices: Vertices,

    /// Geometric primitives that assemble values from the inputs into vertex attribute data.
    #[child]
    pub primitives: Vec<Primitive>,

    /// Arbitrary additional information about this geometry instance and the data it contains.
    ///
    /// For more information about 3rd-party extensions, see the
    /// [crate-level documentation](../index.html#3rd-party-extensions).
    #[child]
    pub extras: Vec<Extra>,
}

impl Mesh {
    /// Returns the source which matches `id`, or `None` if no sources match.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// # use std::fs::File;
    /// # use collaborate::v1_4::Collada;
    /// # let file = File::open("resources/blender_cube.dae").unwrap();
    /// # let document = Collada::read(file).unwrap();
    /// # let library = document.libraries[5].as_library_geometries().unwrap();
    /// let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
    /// let positions_source = mesh.find_source("Cube-mesh-positions");
    /// assert!(positions_source.is_some());
    /// ```
    pub fn find_source<'a>(&'a self, id: &str) -> Option<&'a Source> {
        self.sources.iter().find(|source| source.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "Name_array"]
pub struct NameArray;

/// Declares parametric information for its parent element.
///
/// A functional or programmatical format requires a means for users to specify parametric
/// information. This information represents function parameter (argument) data.
///
/// Material shader programs may contain code representing vertex or pixel programs. These
/// programs require parameters as part of their state information.
///
/// The basic declaration of a parameter describes the name, data type, and value data of the
/// parameter. That parameter name identifies it to the function or program. The parameter type
/// indicates the encoding of its value.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "param"]
pub struct Param {
    /// The name of the parameter.
    #[attribute]
    pub name: Option<String>,

    /// The subidentifier of this parameter.
    ///
    /// This value is unique within the scope of the parent element.
    #[attribute]
    pub sid: Option<String>,

    /// The type of the value data.
    ///
    /// Must be understood by the application consuming the COLLADA document.
    #[attribute]
    #[name = "type"]
    pub data_type: Option<String>,

    /// The user-defined meaning of the parameter.
    #[attribute]
    pub semantic: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Polygon<'a> {
    len: usize,
    chunks: ::std::slice::Chunks<'a, usize>,
}

impl<'a> Polygon<'a> {
    pub fn iter(&self) -> PolygonIter<'a> {
        PolygonIter { chunks: self.chunks.clone() }
    }

    /// Returns the number of vertices in this polygon.
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<'a> ::std::iter::IntoIterator for Polygon<'a> {
    type Item = Vertex<'a>;
    type IntoIter = PolygonIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PolygonIter { chunks: self.chunks }
    }
}

impl<'a> ::std::iter::IntoIterator for &'a Polygon<'a> {
    type Item = Vertex<'a>;
    type IntoIter = PolygonIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PolygonIter { chunks: self.chunks.clone() }
    }
}

pub struct PolygonIter<'a> {
    chunks: ::std::slice::Chunks<'a, usize>,
}

impl<'a> ::std::iter::Iterator for PolygonIter<'a> {
    type Item = Vertex<'a>;

    fn next(&mut self) -> Option<Vertex<'a>> {
        self.chunks.next().map(|attributes| Vertex { attributes })
    }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "polygons"]
pub struct Polygons;

/// A list of polygons that are not necessarily triangles.
///
/// Provides the information needed for a mesh to bind vertex attributes together and then
/// organize those vertices into individual polygons. `Polylist` provides functionality for
/// iterating over the polygons it represents.
///
/// # Examples
///
/// Iterate over all of the polygons in a polylist, then iterate over each vertex in each polygon:
///
/// ```
/// # #![allow(unused_variables)]
/// # use std::fs::File;
/// # use collaborate::v1_4::Collada;
/// # let file = File::open("resources/blender_cube.dae").unwrap();
/// # let document = Collada::read(file).unwrap();
/// # let library = document.libraries[5].as_library_geometries().unwrap();
/// # let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
/// let polylist = mesh.primitives[0].as_polylist().unwrap();
/// for polygon in polylist {
///     println!("Vertices in polygon: {}", polygon.len());
///     for vertex in polygon {
///         println!("{:?}", vertex);
///         for attribute in vertex {
///             for input in polylist.inputs_for_offset(attribute.offset) {
///                 println!(
///                     "Attribute {:?} indexes into {:?}",
///                     attribute,
///                     input,
///                 );
///             }
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "polylist"]
pub struct Polylist {
    /// A human-friendly name for this polylist.
    ///
    /// Has no semantic meaning.
    #[attribute]
    pub name: Option<String>,

    /// The number of polygon primitives in the polylist.
    #[attribute]
    pub count: usize,

    /// The name of the material associated with this polylist.
    ///
    /// This name is bound to a material at the time of instantiaion. See [`InstanceGeometry`]
    /// and [`BindMaterial`].
    ///
    /// If `None`, then the lighting and shading results are appplication-defined.
    ///
    /// [`InstanceGeometry`]: ./struct.InstanceGeometry.html
    /// [`BindMaterial`]: ./struct.BindMaterial.html
    #[attribute]
    pub material: Option<String>,

    /// The input data for the polylist.
    #[child]
    pub inputs: Vec<SharedInput>,

    /// A list of integers, each specifying the number of vertices for one polygon in the polylist.
    #[child]
    pub vcount: Option<VCount>,

    /// A list of integers that specify the vertex attributes as indexes into the inputs.
    #[child]
    pub primitives: Option<Primitives>,

    /// Arbitrary additional information about this polylist and the data it contains.
    ///
    /// For more information about 3rd-party extensions, see the
    /// [crate-level documentation](../index.html#3rd-party-extensions).
    #[child]
    pub extras: Vec<Extra>,
}

impl Polylist {
    /// Returns an iterator over the polygons in the polylist.
    pub fn iter<'a>(&'a self) -> PolylistIter<'a> {
        // Determine the number of indices that are used for each vertex. Generally, we expect this to
        // be the same as the number of inputs (e.g. if there's an input for position and an input
        // for normal, then we'd expect there to be 2 indices for each vertex), but the COLLADA spec
        // allows multiple inputs to share an offset, effectively reducing the number of indices
        // needed for each vertex. To account for this, we look for the largest offset used by the
        // inputs, which should tell us consistently how many unique offsets there are.
        // TODO: How do we handle a polylist with no inputs? Probably return no polygons.
        let largest_offset = self.inputs.iter()
            .map(|input| input.offset)
            .max()
            .unwrap();

        PolylistIter {
            polylist: self,
            num_indices_per_vertex: largest_offset + 1,
            vcount_iter: self.vcount.as_ref().unwrap().iter(),
            verts_so_far: 0,
        }
    }

    /// Returns the number of polygons in the polylist.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns an iterator yielding all inputs that match `offset`.
    ///
    /// When matching a vertex attribute to an input, the attribute's offset is matched against
    /// the input's offset. It's possible for multiple inputs to share the same offset, so this
    /// method provides an easy way to iterate over all inputs with a given offset.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// # use std::fs::File;
    /// # use collaborate::v1_4::Collada;
    /// # let file = File::open("resources/blender_cube.dae").unwrap();
    /// # let document = Collada::read(file).unwrap();
    /// # let library = document.libraries[5].as_library_geometries().unwrap();
    /// # let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
    /// let polylist = mesh.primitives[0].as_polylist().unwrap();
    /// for polygon in polylist {
    ///     println!("Vertices in polygon: {}", polygon.len());
    ///     for vertex in polygon {
    ///         println!("{:?}", vertex);
    ///         for attribute in vertex {
    ///             for input in polylist.inputs_for_offset(attribute.offset) {
    ///                 println!(
    ///                     "Attribute {:?} indexes into {:?}",
    ///                     attribute,
    ///                     input,
    ///                 );
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn inputs_for_offset<'a>(&'a self, offset: usize) -> InputsForOffset<'a> {
        InputsForOffset {
            inputs: self.inputs.iter(),
            offset,
        }
    }
}

impl<'a> ::std::iter::IntoIterator for &'a Polylist {
    type Item = Polygon<'a>;
    type IntoIter = PolylistIter<'a>;

    fn into_iter(self) -> PolylistIter<'a> {
        self.iter()
    }
}

pub struct PolylistIter<'a> {
    polylist: &'a Polylist,
    num_indices_per_vertex: usize,
    vcount_iter: ::std::slice::Iter<'a, usize>,
    verts_so_far: usize,
}

impl<'a> ::std::iter::Iterator for PolylistIter<'a> {
    type Item = Polygon<'a>;

    fn next(&mut self) -> Option<Polygon<'a>> {
        let primitives = match self.polylist.primitives {
            Some(ref primitives) => primitives,
            None => return None,
        };

        self.vcount_iter.next()
            .map(|&num_verts| {
                let indices = &primitives[self.verts_so_far * self.num_indices_per_vertex .. (self.verts_so_far + num_verts) * self.num_indices_per_vertex];
                self.verts_so_far += num_verts;
                Polygon {
                    len: num_verts,
                    chunks: indices.chunks(self.num_indices_per_vertex),
                }
            })
    }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
pub enum Primitive {
    Lines(Lines),
    Linestrips(Linestrips),
    Polygons(Polygons),
    Polylist(Polylist),
    Triangles(Triangles),
    Trifans(Trifans),
    Tristrips(Tristrips),
}

impl Primitive {
    pub fn as_polylist(&self) -> Option<&Polylist> {
        match *self {
            Primitive::Polylist(ref polylist) => Some(polylist),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "p"]
pub struct Primitives {
    #[text]
    data: Vec<usize>,
}

impl ::std::ops::Deref for Primitives {
    type Target = [usize];

    fn deref(&self) -> &[usize] { &*self.data }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "scene"]
pub struct Scene;

/// Declares the input semantic of a data source and connects a consumer of that source.
///
/// `SharedInput` declares the input connection to a data source that a consumer requires. A data
/// source is a container of raw data that lacks semantic meaning, so that the data can be
/// reused within the document. To use the data, a consumer declares a connection to it with the
/// desired semantic information.
///
/// In COLLADA, all inputs are driven by index values. A consumer samples an input by supplying
/// an index value to an input. Some consumers have multiple inputs that can share the same index
/// values. Inputs that have the same `offset` value are driven by the same index value from the
/// consumer. This is an optimization that reduces the total number of indexes that the consumer
/// must store. These inputs are described in this section as shared inputs but otherwise
/// operate in the same manner as unshared inputs.
///
/// # Common Semantics
///
/// | Value of `semantic` | Description                                                |
/// | ------------------- | ---------------------------------------------------------- |
/// | `"BINORMAL"`        | Geometric binormal (bitangent) vector.                     |
/// | `"COLOR"`           | Color coordinate vector. Color inputs are RGB.             |
/// | `"CONTINUITY"`      | Continuity constraint at the control vertex (CV). See also "Curve Interpolation" in Chapter 4 of the COLLADA spec.    |
/// | `"IMAGE"`           | Raster or MIP-level input.                                 |
/// | `"INPUT"`           | Sampler input. See also "Curve Interpolation" in Chapter 4 of the COLLADA spec. |
/// | `"IN_TANGENT"`      | Tangent vector for preceding control point. See also "Curve Interpolation" in Chapter 4 of the COLLADA spec. |
/// | `"INTERPOLATION"`   | Sampler interpolation type. See also "Curve Interpolation" in Chapter 4 of the COLLADA spec. |
/// | `"INV_BIND_MATRIX"` | Inverse of location-to-world matrix.                       |
/// | `"JOIN"`            | Skin influence identifier.                                 |
/// | `"LINEAR_STEPS"`    | Number of piece-wise linear approximation steps to use for the spline segment that follows this CV. See also "Curve Interpolation" in Chapter 4 of the COLLADA spec. |
/// | `"MORPH_TARGET"`    | Morph targets for mesh morphing.                           |
/// | `"MORPH_WEIGHT"`    | Weights for mesh morphing.                                 |
/// | `"NORMAL"`          | Normal vector.                                             |
/// | `"OUTPUT"`          | Sampler output. See also "Curve Interpolation" in Chapter 4 of the COLLADA spec. |
/// | `"OUT_TANGENT"`     | Tangent vector for succeeding control point. See also "Curve Interpolation" in Chapter 4 fo the COLLADA spec. |
/// | `"POSITION"`        | Geometric coordinate vector. See also "Curve Interpolation" in Chapter 4 of the COLLADA spec. |
/// | `"TANGENT"`         | Geometric tangent vector.                                  |
/// | `"TEXBINORMAL"`     | Texture binormal (bitangent) vector.                       |
/// | `"TEXCOORD"`        | Texture coordinate vector.                                 |
/// | `"TEXTANGENT"`      | Texture tangent vector.                                    |
/// | `"UV"`              | Generic parameter vector.                                  |
/// | `"VERTEX"`          | Mesh vertex.                                               |
/// | `"WEIGHT"`          | Skin influence weighting value.                            |
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "input"]
pub struct SharedInput {
    /// The offset into the list of indices provided by the parent object.
    ///
    /// If two `SharedInput` instances share the same `offset` value, they are indexed the same.
    /// This is a simple form of compression for the list of indices and also defines the order
    /// in which inputs are used.
    #[attribute]
    pub offset: usize,

    /// The user-defined meaning of the input connnection.
    ///
    /// See the type-level documentation for a [list of common semantic values](#common-semantics).
    #[attribute]
    pub semantic: String,

    /// The location of the data source.
    #[attribute]
    pub source: UriFragment,

    /// Which inputs to group as a single set.
    ///
    /// This is helpful when multiple inputs share the same semantic.
    #[attribute]
    pub set: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "source"]
pub struct Source {
    #[attribute]
    pub id: String,

    #[attribute]
    pub name: Option<String>,

    #[child]
    pub asset: Option<Asset>,

    #[child]
    pub array: Option<Array>,

    #[child]
    pub technique_common: Option<SourceTechniqueCommon>,

    #[child]
    pub techniques: Vec<Technique>,
}

impl Source {
    // Returns the [`Accessor`] in the source's `technique_common` member.
    pub fn common_accessor(&self) -> Option<&Accessor> {
        self.technique_common
            .as_ref()
            .map(|technique| &technique.accessor)
    }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "technique_common"]
pub struct SourceTechniqueCommon {
    #[child]
    pub accessor: Accessor,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "spline"]
pub struct Spline;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "triangles"]
pub struct Triangles;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "trifans"]
pub struct Trifans;

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "tristrips"]
pub struct Tristrips;

/// Declares the input semantic of a data source and connects a consumer of that source.
///
/// Declares the input connection to a data source that a consumer requires. A data
/// source is a container of raw data that lacks semantic meaning, so that the data can be
/// reused within the document. To use the data, a consumer declares a connection to it with the
/// desired semantic information.
///
/// In COLLADA, all inputs are driven by index values. A consumer samples an input by supplying
/// an index value to an input. Some consumers have multiple inputs that can share the same index
/// values. Inputs that have the same `offset` value are driven by the same index value from the
/// consumer. This is an optimization that reduces the total number of indexes that the consumer
/// must store. These inputs are described in this section as shared inputs but otherwise
/// operate in the same manner as unshared inputs.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "input"]
pub struct UnsharedInput {
    /// The user-defined meaning of the input connnection.
    ///
    /// See [`SharedInput`] for a list of common semantic values.
    ///
    /// [`SharedInput`]: ./struct.SharedInput.html
    #[attribute]
    pub semantic: String,

    /// The location of the data source.
    #[attribute]
    pub source: UriFragment,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "vcount"]
pub struct VCount {
    #[text]
    data: Vec<usize>,
}

impl ::std::ops::Deref for VCount {
    type Target = [usize];

    fn deref(&self) -> &[usize] { &*self.data }
}

/// A single vertex in a polygon.
///
/// A vertex is composed of one or more attributes. You can use `Vertex` to iterate over a list
/// of [`VertexAttribute`] objects representing the attributes of the vertex.
///
/// # Examples
///
/// ```
/// # #![allow(unused_variables)]
/// # use std::fs::File;
/// # use collaborate::v1_4::Collada;
/// # let file = File::open("resources/blender_cube.dae").unwrap();
/// # let document = Collada::read(file).unwrap();
/// # let library = document.libraries[5].as_library_geometries().unwrap();
/// # let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
/// let polylist = mesh.primitives[0].as_polylist().unwrap();
/// for polygon in polylist {
///     for vertex in polygon {
///         for attribute in vertex {
///             println!(
///                 "Input offset: {}, attribute index: {}",
///                 attribute.offset,
///                 attribute.index,
///             );
///         }
///     }
/// }
/// ```
///
/// [`VertexAttribute`]: ./struct.VertexAttribute.html
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex<'a> {
    attributes: &'a [usize],
}

impl<'a> Vertex<'a> {
    /// Returns an iterator over the attributes in the vertex.
    ///
    /// # Examples
    /// ```
    /// # #![allow(unused_variables)]
    /// # use std::fs::File;
    /// # use collaborate::v1_4::*;
    /// # let file = File::open("resources/blender_cube.dae").unwrap();
    /// # let document = Collada::read(file).unwrap();
    /// # let library = document.libraries[5].as_library_geometries().unwrap();
    /// # let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
    /// # let polylist = mesh.primitives[0].as_polylist().unwrap();
    /// # let polygon = polylist.iter().next().unwrap();
    /// let vertex = polygon.iter().next().unwrap();
    /// let mut iter = vertex.iter();
    /// assert_eq!(Some(VertexAttribute { index: 0, offset: 0 }), iter.next());
    /// assert_eq!(Some(VertexAttribute { index: 0, offset: 1 }), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    pub fn iter(&self) -> VertexIter<'a> {
        VertexIter {
            iter: self.attributes.iter(),
            offset: 0,
        }
    }
}

impl<'a> ::std::iter::IntoIterator for Vertex<'a> {
    type Item = VertexAttribute;
    type IntoIter = VertexIter<'a>;

    fn into_iter(self) -> VertexIter<'a> { self.iter() }
}

impl<'a> ::std::iter::IntoIterator for &'a Vertex<'a> {
    type Item = VertexAttribute;
    type IntoIter = VertexIter<'a>;

    fn into_iter(self) -> VertexIter<'a> { self.iter() }
}

/// Represents a single attribute of a vertex.
///
/// A vertex attribute has two properties:
///
/// * An offset, used to determine which input(s) this attribute references.
/// * An index, which is used to index into the data specified by the referenced input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    /// The index within the relevant source array which has this attribute's value.
    pub index: usize,

    /// The offset of the attribute.
    ///
    /// This value will match the `offset` member of one or more inputs ([`SharedInput`] or
    /// [`UnsharedInput`]). If the attribute matches more than one input, then the attribute
    /// indexes into all of the inputs it matches. Therefore, a single `VertexAttribute` can
    /// map to multiple actual vertex attributes.
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct VertexIter<'a> {
    iter: ::std::slice::Iter<'a, usize>,
    offset: usize,
}

impl<'a> ::std::iter::Iterator for VertexIter<'a> {
    type Item = VertexAttribute;

    fn next(&mut self) -> Option<VertexAttribute> {
        self.iter.next().map(|&index| {
            let attribute = VertexAttribute { index, offset: self.offset };
            self.offset += 1;
            attribute
        })
    }
}

/// Declares the attributes and identity of mesh-vertices.
///
/// Mesh-vertices represent the position (identity) of the vertices comprising the mesh and other
/// vertex attributes that are invariant to tessellation.
#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "vertices"]
pub struct Vertices {
    /// A unique identifier of the vertices instance.
    ///
    /// This value is unique within the document.
    #[attribute]
    pub id: String,

    /// The name of the vertices instance.
    #[attribute]
    pub name: Option<String>,

    /// The input data for the vertices.
    ///
    /// There will be at least one element in `inputs`, and one input will specify the
    /// `"POSITION"` semantic.
    #[child]
    #[required]
    pub inputs: Vec<UnsharedInput>,

    /// Arbitrary additional data about the vertices.
    #[child]
    pub extras: Vec<Extra>,
}
