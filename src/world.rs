use std::error::Error;

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
    pub blocks: Vec<Option<Block>>,
    pub textures: FxHashMap<String, TextureHandle>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl World {
    pub fn flatten_coords(&self, x: usize, y: usize, z: usize) -> usize {
        x + self.width as usize * (y + z * self.height as usize)
    }

    pub fn get_block(&self, x: u32, y: u32, z: u32) -> Result<Block, Box<dyn Error>> {
        if let Some(block) = self
            .blocks
            .get(self.flatten_coords(x as usize, y as usize, z as usize))
        {
            return block.ok_or("err".into());
        }
        Err("".into())
    }

    pub fn get_block_mut(&mut self, x: u32, y: u32, z: u32) -> Result<&mut Block, Box<dyn Error>> {
        // let (x, y, z) = self.world_coords_to_unsigned(x, y, z)?;
        let index = self.flatten_coords(x as usize, y as usize, z as usize);
        if let Some(block) = self.blocks.get_mut(index) {
            return block.as_mut().ok_or_else(|| "".into());
        }
        Err("".into())
    }

    pub fn new(width: u32, height: u32, depth: u32, perlin_threshold: f32) -> Self {
        let p = Perlin::new(1);
        let mut blocks = vec![];
        for x in 0..width {
            for y in 0..height {
                for z in 0..depth {
                    let val = p.get([x as f64 / 16.0, y as f64 / 16.0, z as f64 / 16.0]);
                    #[allow(clippy::overly_complex_bool_expr)]
                    if val > perlin_threshold as f64 {
                        blocks.push(Some(Block {
                            position: vec3(x as f32, -5. - z as f32, y as f32),
                            rotation: Quat::default(),
                            block_type: BlockType::random(),
                            visible: true,
                        }));
                    } else {
                        blocks.push(None);
                    }
                }
            }
        }

        let mut this = Self {
            blocks,
            textures: FxHashMap::default(),
            width,
            height,
            depth,
        };

        this.block_visibility();

        this
    }

    fn block_visibility(&mut self) -> Result<(), Box<dyn Error>> {
        // determine which blocks are visible
        for x in 1..self.width - 1 {
            for y in 1..self.height - 1 {
                for z in 1..self.depth - 1 {
                    if self.get_block(x, y, z).is_ok() {
                        let left = self.get_block(x - 1, y, z);
                        let right = self.get_block(x + 1, y, z);
                        let front = self.get_block(x, y, z + 1);
                        let back = self.get_block(x, y, z - 1);
                        let top = self.get_block(x, y + 1, z);
                        let bottom = self.get_block(x, y - 1, z);
                        let surrounding_blocks = [left, right, front, back, top, bottom];
                        if surrounding_blocks.iter().all(|result| result.is_ok()) {
                            self.get_block_mut(x, y, z).unwrap().visible = false;
                        }
                    }
                }
            }
        }
        println!(
            "invisible blocks {}",
            self.blocks
                .iter()
                .flatten()
                .filter(|block| !block.visible)
                .count()
        );
        println!(
            "visible blocks {}",
            self.blocks
                .iter()
                .flatten()
                .filter(|block| block.visible)
                .count()
        );
        Ok(())
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
            .flatten()
            .filter(|block| block.visible)
            .for_each(|block| block.draw(renderer, self));
    }
}

#[cfg(test)]
mod tests {
    use super::World;

    #[test]
    fn flat_index_test() {
        let world = World::new(3, 3, 3, -9999.0); // a solid cube
        let mut counter = 0;
        for z in 0..3 {
            for y in 0..3 {
                for x in 0..3 {
                    assert!(
                        world.flatten_coords(x, y, z) == counter,
                        "{x}, {y}, {z} yielded {} instead of {counter}",
                        world.flatten_coords(x, y, z)
                    );
                    counter += 1;
                }
            }
        }
    }

    #[test]
    fn interior_blocks_are_invisible() {
        let world = World::new(3, 3, 3, -9999.0); // a solid cube

        // in a 3x3x3 world we would expect that the middle block is invisible and the rest are visible
        for (idx, block) in world.blocks.iter().enumerate() {
            if idx == 13 {
                assert!(
                    block.unwrap().visible == false,
                    "{idx}th block should've been visible"
                );
            } else {
                assert!(
                    block.unwrap().visible == true,
                    "{idx}th block should've been invisible"
                );
            }
        }
    }

    #[test]
    fn interior_blocks_are_invisible_bigger() {
        let world = World::new(4, 4, 4, -9999.0); // a solid cube

        // in a 3x3x3 world we would expect that the middle block is invisible and the rest are visible
        for (idx, block) in world.blocks.iter().enumerate() {
            if vec![21, 22, 25, 26, 37, 38, 41, 42]
                .iter()
                .any(|x| *x == idx)
            {
                assert!(
                    !block.unwrap().visible,
                    "{idx}th block should've been visible"
                );
            } else {
                assert!(
                    block.unwrap().visible,
                    "{idx}th block should've been invisible"
                );
            }
        }
    }
}
