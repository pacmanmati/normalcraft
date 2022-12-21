use fxhash::FxHashMap;
use glam::{vec3, Quat, Vec3};
use image::DynamicImage;

use crate::{
    instance::Instance,
    renderer::{v, Drawable, Renderer, Vertex},
    texture::TextureHandle,
};

#[derive(Clone, Copy)]
enum BlockType {
    Dirt,
    Cobble,
    Stone,
    Water,
    Sand,
}

impl BlockType {
    pub fn random() -> Self {
        let r = rand::random::<f32>();
        r.into()
    }
}

impl From<f32> for BlockType {
    fn from(f: f32) -> BlockType {
        match f {
            x if (0.0..0.33).contains(&x) => BlockType::Dirt,
            x if (0.33..0.66).contains(&x) => BlockType::Cobble,
            x if (0.66..1.0).contains(&x) => BlockType::Sand,
            _ => BlockType::Dirt,
        }
    }
}

// impl From<String> for BlockType {
//     fn from(s: String) -> BlockType {
//         match s.as_str() {
//             "dirt" => BlockType::Dirt,
//             "cobble" => BlockType::Cobble,
//             "stone" => BlockType::Cobble,
//             "sand" => BlockType::Cobble,
//             _ => BlockType::Dirt,
//         }
//     }
// }

impl From<&str> for BlockType {
    fn from(s: &str) -> BlockType {
        match s {
            "dirt" => BlockType::Dirt,
            "cobble" => BlockType::Cobble,
            "stone" => BlockType::Stone,
            "sand" => BlockType::Sand,
            _ => BlockType::Dirt,
        }
    }
}

impl<'a> From<BlockType> for &'a str {
    fn from(ty: BlockType) -> &'a str {
        match ty {
            BlockType::Dirt => "dirt",
            BlockType::Cobble => "cobble",
            BlockType::Stone => "stone",
            BlockType::Sand => "sand",
            BlockType::Water => "water",
        }
    }
}

// we want cube vertices (textured) and indices
// this diagram's vertices are unrelated...
//     .v0--------.v1,9,12
//    /|         /|
//   / |        / |
//  /  |       /  |
// .v2--------.v3 |
// |   |v6____|__ .v7,13
// |  /       |  /
// | /        | /
// |/         |/
// .v4--------.v5

pub fn cube_vertices() -> Vec<Vertex> {
    vec![
        v(-0.5, 0.5, -0.5, 1.0 / 3.0, 0.0),   // v0
        v(0.5, 0.5, -0.5, 2.0 / 3.0, 0.0),    // v1 --
        v(-0.5, 0.5, 0.5, 1.0 / 3.0, 0.25),   // v2
        v(0.5, 0.5, 0.5, 2.0 / 3.0, 0.25),    // v3 --
        v(-0.5, -0.5, 0.5, 1.0 / 3.0, 0.5),   // v4
        v(0.5, -0.5, 0.5, 2.0 / 3.0, 0.5),    // v5 --
        v(-0.5, -0.5, -0.5, 1.0 / 3.0, 0.75), // v6
        v(0.5, -0.5, -0.5, 2.0 / 3.0, 0.75),  // v7 --
        v(-0.5, 0.5, -0.5, 1.0 / 3.0, 1.0),   // v8
        v(0.5, 0.5, -0.5, 2.0 / 3.0, 1.0),    // v9
        //
        v(-0.5, 0.5, -0.5, 0.0, 0.25), // v10
        v(-0.5, -0.5, -0.5, 0.0, 0.5), // v11
        //
        v(0.5, 0.5, -0.5, 1.0, 0.25), // v12 --
        v(0.5, -0.5, -0.5, 1.0, 0.5), // v13 --
    ]
}

// uv-unwrapped cube:
//
//          v0----v1
//          |  f0  |
//          |      |
//  v10----v2-----v3----v12
//   |  f4  |  f1  |  f5  |
//   |      |      |      |
//  v11----v4-----v5----v13
//          |  f2  |
//          |      |
//          v6----v7
//          |  f3  |
//          |      |
//          v8----v9

pub fn cube_indices() -> Vec<u16> {
    vec![
        0, 3, 1, 0, 2, 3, // f0
        2, 5, 3, 2, 4, 5, // f1
        4, 7, 5, 4, 6, 7, // f2
        6, 9, 7, 6, 8, 9, // f3
        10, 4, 2, 10, 11, 4, // f4
        3, 13, 12, 3, 5, 13, // f5
    ]
}

pub struct Block {
    position: Vec3,
    rotation: Quat,
    block_type: BlockType,
}

// drawing one individual instance makes little sense...
// the renderer could batch instances
// allowing us to bind buffers once and do only 1 draw call
impl Drawable for Block {
    fn draw(&self, renderer: &mut Renderer, world: &World) {
        renderer.queue_draw(self, world);
    }
    fn vertices(&self) -> Vec<Vertex> {
        cube_vertices()
    }

    fn indices(&self) -> Vec<u16> {
        cube_indices()
    }

    fn instance(&self, world: &World) -> Instance {
        let tex_name = std::convert::Into::<&str>::into(self.block_type);
        let texture = world.get_texture(tex_name);
        Instance::new(self.position, self.rotation, texture)
    }
}

// the world will consist of blocks and entities
pub struct World {
    pub blocks: Vec<Block>,
    pub textures: FxHashMap<String, TextureHandle>,
}

impl World {
    pub fn new() -> Self {
        let mut blocks = vec![];
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    blocks.push(Block {
                        position: vec3(x as f32, -5. - z as f32, y as f32),
                        rotation: Quat::default(),
                        block_type: BlockType::random(),
                    })
                }
            }
        }
        Self {
            blocks,
            textures: FxHashMap::default(),
        }
    }

    pub fn setup_textures(
        &mut self,
        renderer: &mut Renderer,
        textures: Vec<(String, DynamicImage)>,
    ) {
        // how do we even identify these images?
        // at some point we read the files (./assets/dirt.png)
        // do we assign a string label and then create a mapping of String <-> BlockType ?
        let handles: FxHashMap<String, TextureHandle> = textures
            .into_iter()
            .map(|(label, tex)| (label, renderer.register_texture(tex)))
            .collect();
        self.textures = handles;
    }

    pub fn get_texture(&self, tex_name: &str) -> TextureHandle {
        *self
            .textures
            .get(tex_name)
            .unwrap_or_else(|| panic!("No texture found for {tex_name} in {:?}", self.textures))
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        self.blocks.iter().for_each(|x| x.draw(renderer, self));
    }
}
