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

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "accessor"]
pub struct Accessor {
    #[attribute]
    pub count: usize,

    #[attribute]
    #[optional_with_default = "0"]
    pub offset: usize,

    #[attribute]
    pub source: AnyUri,

    #[attribute]
    #[optional_with_default = "1"]
    pub stride: usize,

    #[child]
    pub params: Vec<Param>,
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
    pub fn as_mesh(&self) -> Option<&Mesh> {
        match *self {
            GeometricElement::Mesh(ref mesh) => Some(mesh),
            _ => None,
        }
    }
}

/// Describes the visual shape and appearance of an object in a scene.
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
    /// [`Source`]: ./struct.Source.html
    #[child]
    #[required]
    pub sources: Vec<Source>,

    #[child]
    pub vertices: Vertices,

    #[child]
    pub primitives: Vec<Primitive>,

    /// Arbitrary additional information about this geometry instance and the data it contains.
    ///
    /// For more information about 3rd-party extensions, see the
    /// [crate-level documentation](../index.html#3rd-party-extensions).
    #[child]
    pub extras: Vec<Extra>,
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "Name_array"]
pub struct NameArray;


#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "param"]
pub struct Param {
    #[attribute]
    pub name: Option<String>,

    #[attribute]
    pub sid: Option<String>,

    #[attribute]
    #[name = "type"]
    pub data_type: Option<String>,

    #[attribute]
    pub semantic: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Polygon<'a> {
    len: usize,
    chunks: ::std::slice::Chunks<'a, usize>,
}

impl<'a> Polygon<'a> {
    pub fn vertices(&self) -> ::std::slice::Chunks<'a, usize> {
        self.chunks.clone()
    }

    /// Returns the number of vertices in this polygon.
    pub fn len(&self) -> usize {
        self.len
    }
}

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "polygons"]
pub struct Polygons;

/// A list of polygons that are not necessarily triangles.
///
/// Provides the information needed for a mesh to bind vertex attributes together and then
/// organize those vertices into individual polygons.
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

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "input"]
pub struct SharedInput {
    #[attribute]
    pub offset: usize,

    #[attribute]
    pub semantic: String,

    #[attribute]
    pub source: UriFragment,

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

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "input"]
pub struct UnsharedInput {
    #[attribute]
    pub semantic: String,

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

#[derive(Debug, Clone, PartialEq, ColladaElement)]
#[name = "vertices"]
pub struct Vertices {
    #[attribute]
    pub id: String,

    #[attribute]
    pub name: Option<String>,

    #[child]
    #[required]
    pub inputs: Vec<UnsharedInput>,

    #[child]
    pub extras: Vec<Extra>,
}
