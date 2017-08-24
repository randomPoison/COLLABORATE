use {AnyUri, DateTime, Error, ErrorKind, Extra, Result, Unit, UpAxis, utils, v1_5};
use utils::*;
use xml::common::Position;

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

impl Into<v1_5::Collada> for Collada {
    fn into(self) -> v1_5::Collada {
        v1_5::Collada {
            version: self.version,
            base_uri: self.base,
            asset: self.asset.into(),
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
    pub unit: Unit,

    #[child]
    pub up_axis: UpAxis,
}

/*
impl ColladaElement for Asset {
    fn name_test(name: &str) -> bool {
        name == "asset"
    }

    fn parse_element<R>(
        reader: &mut EventReader<R>,
        element_start: ElementStart,
    ) -> Result<Asset>
    where
        R: Read,
    {
        utils::verify_attributes(reader, "asset", element_start.attributes)?;

        let mut contributors = Vec::default();
        let mut created = None;
        let mut keywords = Vec::new();
        let mut modified = None;
        let mut revision = None;
        let mut subject = None;
        let mut title = None;
        let mut unit = None;
        let mut up_axis = None;

        ElementConfiguration {
            name: "asset",
            children: &mut [
                ChildConfiguration {
                    name: &|name| { name == "contributor" },
                    occurrences: Many,

                    action: &mut |reader, start_element| {
                        let contributor = Contributor::parse_element(reader, start_element)?;
                        contributors.push(contributor);
                        Ok(())
                    },

                    add_names: &|names| { names.push("contributor"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "created" },
                    occurrences: Required,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "created", start_element.attributes)?;
                        created = utils::optional_text_contents(reader, "created")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("created"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "keywords" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "keywords", start_element.attributes)?;
                        if let Some(keywords_string) = utils::optional_text_contents::<_, String>(reader, "keywords")? {
                            keywords = keywords_string
                                .split_whitespace()
                                .map(Into::into)
                                .collect();
                        }
                        Ok(())
                    },

                    add_names: &|names| { names.push("keywords"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "modified" },
                    occurrences: Required,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "modified", start_element.attributes)?;
                        modified = utils::optional_text_contents(reader, "modified")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("modified"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "revision" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "revision", start_element.attributes)?;
                        revision = utils::optional_text_contents(reader, "revision")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("revision"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "subject" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "subject", start_element.attributes)?;
                        subject = utils::optional_text_contents(reader, "subject")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("subject"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "title" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        utils::verify_attributes(reader, "title", start_element.attributes)?;
                        title = utils::optional_text_contents(reader, "title")?;
                        Ok(())
                    },

                    add_names: &|names| { names.push("title"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "unit" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        unit = Some(Unit::parse_element(reader, start_element)?);
                        Ok(())
                    },

                    add_names: &|names| { names.push("unit"); },
                },

                ChildConfiguration {
                    name: &|name| { name == "up_axis" },
                    occurrences: Optional,

                    action: &mut |reader, start_element| {
                        up_axis = Some(UpAxis::parse_element(reader, start_element)?);
                        Ok(())
                    },

                    add_names: &|names| { names.push("up_axis"); },
                },
            ],
        }.parse_children(reader)?;

        Ok(Asset {
            contributors: contributors,
            created: created.expect("Required element was not found"),
            keywords: keywords,
            modified: modified.expect("Required element was not found"),
            revision: revision,
            subject: subject,
            title: title,
            unit: unit.unwrap_or_default(),
            up_axis: up_axis.unwrap_or_default(),
        })
    }

    fn add_names(names: &mut Vec<&'static str>) {
        names.push("asset");
    }
}
*/

impl Into<v1_5::Asset> for Asset {
    fn into(self) -> v1_5::Asset {
        v1_5::Asset {
            contributors: self.contributors.into_iter().map(Into::into).collect(),
            coverage: None,
            created: self.created,
            keywords: self.keywords,
            modified: self.modified,
            revision: self.revision,
            subject: self.subject,
            title: self.title,
            unit: self.unit,
            up_axis: self.up_axis,
            extras: Vec::default(),
        }
    }
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

impl Into<v1_5::Contributor> for Contributor {
    fn into(self) -> v1_5::Contributor {
        v1_5::Contributor {
            author: self.author,
            author_email: None,
            author_website: None,
            authoring_tool: self.authoring_tool,
            comments: self.comments,
            copyright: self.copyright,
            source_data: self.source_data,
        }
    }
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
