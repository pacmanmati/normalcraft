use fxhash::FxHashMap;
use glam::{vec3, Quat, Vec3};
use image::DynamicImage;
use noise::{NoiseFn, Perlin};

use crate::{
    instance::Instance,
    renderer::{v, Drawable, Renderer, Vertex},
    texture::TextureHandle,
};

#[derive(Clone, Copy, Default)]
enum BlockType {
    #[default]
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
            x if (0.0..0.2).contains(&x) => BlockType::Dirt,
            x if (0.2..0.4).contains(&x) => BlockType::Cobble,
            x if (0.4..0.6).contains(&x) => BlockType::Sand,
            x if (0.6..0.8).contains(&x) => BlockType::Stone,
            x if (0.8..1.0).contains(&x) => BlockType::Water,
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

#[derive(Default, Clone, Copy)]
pub struct Block {
    position: Vec3,
    rotation: Quat,
    block_type: BlockType,
    visible: bool,
}

// drawing one individual instance makes little sense...
// the renderer could batch instances
// allowing us to bind buffers once and do only 1 draw call
impl Drawable for Block {
    fn draw(&self, renderer: &mut Renderer, world: &World) {
        renderer.queue_draw(0, self, world);
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
    // pub blocks: [[[Option<Block>; 64]; 64]; 64],
    pub blocks: Vec<Block>,
    pub textures: FxHashMap<String, TextureHandle>,
}

impl World {
    pub fn new() -> Self {
        let p = Perlin::new(1);
        // let mut blocks = [[[None; 64]; 64]; 64];
        let mut blocks = vec![];
        for x in -64..64 {
            for y in -64..64 {
                for z in -64..64 {
                    let val = p.get([x as f64 / 16.0, y as f64 / 16.0, z as f64 / 16.0]);
                    // println!("{val}");
                    if val > 0.0 {
                        blocks.push(Block {
                            position: vec3(x as f32, -5. - z as f32, y as f32),
                            rotation: Quat::default(),
                            block_type: BlockType::random(),
                            visible: false,
                        });
                    }
                }
            }
        }
        Self {
            blocks,
            textures: FxHashMap::default(),
        }
    }

    // for each block, check if there's any air blocks next to it
    // if so mark it as visible
    fn visibility_floodfill(&self) {
        let sz = 32;
        for x in 0..sz {
            for y in 0..sz {
                for z in 0..sz {}
            }
        }
    }

    // method as described here: https://tomcc.github.io/2014/08/31/visibility-1.html
    fn chunk_floodfill(&self, x: i32, y: i32, z: i32, chunk_size: i32) -> u16 {
        // compute which pairs of faces are visible within the chunk using a 3d floodfill
        // we can later use this information to determine whether we can see from one chunk into another
        // we represent this using 15 bits
        // there are 6 faces in a chunk so 6*5/2=15 pairs (avoiding duplicates)
        let faces = 0_u16;

        faces
    }

    // perform an occlusion check on loaded blocks to determine which blocks are visible.
    // method as described here: https://tomcc.github.io/2014/08/31/visibility-2.html
    pub fn occlusion(&mut self) {
        let chunk_size = 16;
        let mut visibility_grid: [[[u16; 10]; 10]; 10] = [[[0_u16; 10]; 10]; 10];
        // let's do a 10x10x10 area of chunks around the player for now
        // we don't need to recompute this unless the blocks change
        for x in -5..5 {
            for y in -5..5 {
                for z in -5..5 {
                    visibility_grid[x as usize][y as usize][z as usize] =
                        self.chunk_floodfill(x, y, z, chunk_size);
                }
            }
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
        self.blocks
            .iter()
            .for_each(|block| block.draw(renderer, self));
    }
}
