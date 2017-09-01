extern crate collaborate;

use ::collaborate::v1_4::*;

static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: Option<[f32; 3]>,
}

fn main() {
    // Load the COLLADA document from the source string.
    let source = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    let document = Collada::from_str(&*source).unwrap();

    // Grab the `<library_geometries>` instance.
    let library = document.libraries[5].as_library_geometries().unwrap();

    // Get the `<mesh>` instance and put together the mesh data.
    let mesh = library.geometries[0].geometric_element.as_mesh().unwrap();
    let polylist = mesh.primitives[0].as_polylist().unwrap();

    for polygon in polylist {
        let mut vertices = Vec::new();
        for vertex in &polygon {
            let mut position = None;
            let mut normal = None;

            // For each of the attributes in the vertex, find the correct input and then grab
            // the vertex data.
            for (offset, &attribute) in vertex.iter().enumerate() {
                println!("offset: {}, index: {}", offset, attribute);

                // TODO: Provide a helper method `inputs_for_offset` to make this less verbose.
                // Doing so is a pain to implement without impl Trait syntax.
                for input in polylist.inputs.iter().filter(|input| input.offset == offset) {
                    println!("{:#?}", input);
                    // Handle the input based on its semantic.
                    match input.semantic.as_ref() {
                        "VERTEX" => {
                            // The "VERTEX" semantic means that this input indexes into all of
                            // sources specified in the `vertices` member of the host mesh.
                            println!("VERTEX input: {:?}", input);
                        }

                        "NORMAL" => {
                            // Find the mesh source identified by the input's `source` within the
                            // parent `Mesh` object.
                            let source = mesh.sources.iter()
                                .find(|source| source.id == input.source.id())
                                .expect("Didn't find a source with a matching ID in the parent mesh");

                            // Retrieve the source's accessor and raw float array. We only support
                            // using floats for position and normal source data, so we ignore
                            // any other type of array source.
                            let accessor = &source.common_accessor().expect("Source has no accessor");
                            let array = source.array.as_ref()
                                .and_then(Array::as_float_array)
                                .expect("Source wasn't a float array");

                            /// Use the accessor to get the normal data for the current vertex.
                            let normal_data = accessor.access(array.data.as_ref(), attribute);

                            // Use the `params` in the accesor to determine which elements in
                            // `normal_data` correspond to the normal's X, Y, and Z components.
                            let mut x = None;
                            let mut y = None;
                            let mut z = None;

                            for (param, &normal_component) in accessor.params.iter().zip(normal_data.iter()) {
                                match param.name.as_ref().map(String::as_str) {
                                    Some("X") => { x = Some(normal_component); }
                                    Some("Y") => { y = Some(normal_component); }
                                    Some("Z") => { z = Some(normal_component); }

                                    // Ignore any unrecognized or unsupported names.
                                    _ => {}
                                }
                            }

                            normal = Some([
                                x.expect("Normal had no X component"),
                                y.expect("Normal had no Y component"),
                                z.expect("Normal had no Z component"),
                            ])
                        }

                        // Ignore any unknown semantics.
                        semantic @ _ => { println!("Ignoring unknown semantic {:?}", semantic); }
                    }
                }
            }

            println!("Normal {:?}", normal);
            vertices.push(Vertex {
                position: position.expect("Vertex had no position input"),
                normal,
            });
        }

        println!("Vertices in polygon: {:#?}", vertices);
    }
}
