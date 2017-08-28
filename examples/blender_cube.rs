extern crate collaborate;

use ::collaborate::v1_4::*;

static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");

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
            for (offset, attribute) in vertex.iter().enumerate() {
                println!("offset: {}, index: {}", offset, attribute);

                // TODO: Provide a helper method `inputs_for_offset` to make this less verbose.
                // Doing so is a pain to implement without impl Trait syntax.
                for input in polylist.inputs.iter().filter(|input| input.offset == offset) {
                    println!("{:#?}", input);
                }
            }

            vertices.push(Vertex {
                position: position.expect("Vertex had no position input"),
                normal,
            })
        }
    }
}
