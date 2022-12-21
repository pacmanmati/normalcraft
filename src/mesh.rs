struct Mesh {
    data: [u8],
}

impl Mesh {}

struct MeshBuilder<'a> {
    renderer: &'a Renderer,
}

impl<'a> MeshBuilder {
    pub fn new<'a>(renderer: &'a Renderer) -> Self {}
}
