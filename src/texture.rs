use image::DynamicImage;

impl From<&DynamicImage> for Rect {
    fn from(value: &DynamicImage) -> Self {
        Rect {
            x: 0,
            y: 0,
            w: value.width() as i32,
            h: value.height() as i32,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Ord for Rect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.h).cmp(&other.h)
    }
}

impl PartialOrd for Rect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.h.partial_cmp(&other.h)
    }
}

pub type TextureHandle = u32;

pub struct TextureAtlas {
    counter: u32,
    rects: Vec<(Rect, TextureHandle)>,
    pub width: i32,
    pub height: i32,
}

impl TextureAtlas {
    pub fn new() -> Self {
        Self {
            counter: 0,
            // entries: FxHashMap::default(),
            rects: vec![],
            width: 0,
            height: 0,
        }
    }

    pub fn add(&mut self, w: i32, h: i32) -> TextureHandle {
        let handle = self.counter;
        self.counter += 1;
        let rect = Rect { x: 0, y: 0, w, h };
        self.rects.push((rect, handle));
        handle
    }

    pub fn pack(&mut self) {
        // let's go for a fixed width to break on
        let mut x = 0;
        let mut y = 0;
        self.width = 512;
        // sort s.t. the tallest rect is first
        // decreasing rect height means we can place anything
        self.rects.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        // self.rects.reverse();
        let mut max_h = self.rects.first().unwrap().0.h;
        for (rect, _) in self.rects.iter_mut() {
            // bounds check
            if rect.w == 149 && rect.h == 108 {
                println!("this one!");
            }
            println!(
                "{x}, {}, {}, {}, {}",
                rect.x,
                rect.h,
                self.width,
                x + rect.x + rect.w >= self.width
            );
            if x + rect.x + rect.w >= self.width {
                y += max_h;
                x = 0;
                max_h = rect.h;
            }
            // place rect
            rect.x = x;
            rect.y = y;
            // move along
            x += rect.w;
        }
        self.height = y + max_h;
        // println!("{}, {:?}", self.height, self.rects);
    }

    pub fn get_rect(&self, handle: &TextureHandle) -> Option<(Rect, TextureHandle)> {
        self.rects.iter().find(|(_, x)| x == handle).copied()
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
