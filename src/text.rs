use std::ffi::OsStr;

use freetype::{bitmap::PixelMode, face::LoadFlag, Library};
use fxhash::FxHashMap;
use glam::{ivec2, IVec2};
use image::{DynamicImage, GenericImage, Rgba, RgbaImage};

use crate::texture::{Rect, TextureAtlas, TextureHandle};

const CHARS: [char; 26 * 2 + 1] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ',
];

pub struct CharacterMetric {
    pub size: IVec2,
    pub bearing: IVec2,
    pub advance: i32,
}

pub struct Font {
    pub atlas: TextureAtlas,
    glyph_map: FxHashMap<char, TextureHandle>,
    pub tex: DynamicImage,
    pub metrics: FxHashMap<char, CharacterMetric>,
}

impl Font {
    pub fn new<S: AsRef<OsStr>>(path: S, px: u32) -> Self {
        let lib = Library::init().unwrap();
        // load the ttf font at the specified path
        let face = lib.new_face(path, 0).unwrap();
        face.set_pixel_sizes(0, px)
            .unwrap_or_else(|err| panic!("{err}"));
        // initialise an atlas for all glyphs, store an index of char -> TextureHandle
        let mut atlas = TextureAtlas::new();
        let mut bitmaps = vec![];
        let mut glyph_map = FxHashMap::default();
        let mut metrics = FxHashMap::default();
        for char in CHARS {
            face.load_char(char as usize, LoadFlag::RENDER)
                .unwrap_or_else(|err| panic!("Face failed to load char: {char}, err: {err}"));
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();

            let bearing = ivec2(glyph.bitmap_left(), glyph.bitmap_top());
            let size = ivec2(bitmap.width(), bitmap.rows());
            let advance = glyph.advance().x as i32;

            let handle = atlas.add(bitmap.width(), bitmap.rows());
            // println!("{}, {}", bitmap.width(), bitmap.rows());
            glyph_map.insert(char, handle);
            let buffer = bitmap.buffer().to_vec();
            bitmaps.push((buffer, handle, bitmap.pixel_mode().unwrap()));
            metrics.insert(
                char,
                CharacterMetric {
                    size,
                    bearing,
                    advance,
                },
            );
        }

        atlas.pack();

        let mut tex =
            DynamicImage::ImageRgba8(RgbaImage::new(atlas.width as u32, atlas.height as u32));
        for (bitmap, handle, pixel_mode) in bitmaps {
            let (rect, _) = atlas
                .get_rect(&handle)
                .unwrap_or_else(|| panic!("Expected rect for handle {handle}."));
            // what does each u8 of our bitmap buffer represent? that will depend on the pixel mode
            // for now let's assume it's PixelMode::Gray (each u8 is a pixel) and panic otherwise
            assert!(
                pixel_mode == PixelMode::Gray,
                "pixel mode was {pixel_mode:?}",
            );
            let mut row = 0;
            let mut col = 0;
            println!("rect: {rect:?}");
            for pixel in bitmap {
                if col >= rect.w {
                    col = 0;
                    row += 1
                }
                // to convert grayscale to rgb we assume gray = (R+G+B)/3 so R = G = B = gray
                tex.put_pixel(
                    rect.x as u32 + col as u32,
                    rect.y as u32 + row as u32,
                    Rgba([
                        pixel,
                        pixel,
                        pixel,
                        if pixel == 0_u8 { 0_u8 } else { 255_u8 },
                    ]),
                );
                col += 1;
            }
        }

        tex.save("font-bitmap.png")
            .unwrap_or_else(|err| panic!("{err}"));

        Self {
            glyph_map,
            atlas,
            tex,
            metrics,
        }
    }

    pub fn get_char_rect(&self, char: char) -> Rect {
        let handle = self
            .glyph_map
            .get(&char)
            .expect("Couldn't find glyph in glyph map.");
        let (rect, _) = self
            .atlas
            .get_rect(handle)
            .expect("Couldn't find rect in atlas.");
        rect
    }
}
