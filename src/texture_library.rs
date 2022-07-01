use ktx2::Reader;
use std::{collections::HashMap, fs::File, io::Read, path::Path, sync::Arc};
use wgpu::{Device, Queue};

crate::macros::parallel_enum_values! {
    (
        TextureId,
        TEXTURE_PATH_PAIRS,
        str,
    )
    CrabTexture -> "texture/crabdance-seamless-tile.ktx2",
}

// Each texture uses it's own internal texture, view, sampler, and bind group.
// Some of this may be redundent. TODO: reduce redundency in samplers.
pub struct Texture {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    texture_sampler: wgpu::Sampler,
    texture_bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn from_file(device: &Device, queue: &Queue, path: &Path) -> Self {
        let mut file = File::open(path)
            .unwrap_or_else(|e| panic!("failed to open texture file {}: {}", path.display(), e));

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap_or_else(|e| {
            panic!(
                "failed to read contents of texture file into buffer {}: {}",
                path.display(),
                e
            )
        });

        let reader = Reader::new(contents)
            .unwrap_or_else(|e| panic!("failed to parse texture file {}: {}", path.display(), e));

        let header = reader.header();

        assert_eq!(header.format, Some(ktx2::Format::R8G8B8A8_SRGB));
        assert_eq!(header.pixel_depth, 0);
        assert_eq!(header.level_count, 1);
        assert_eq!(header.supercompression_scheme, None);

        let width = header.pixel_width;
        let height = header.pixel_height;

        //let dfd = reader.data_format_descriptors().next();

        let texture_data = reader.levels().next().unwrap();

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * texture_size.width),
                rows_per_image: std::num::NonZeroU32::new(4 * texture_size.height),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout: &&texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        Self {
            texture,
            texture_view,
            texture_sampler,
            texture_bind_group,
        }
    }
}

pub struct TextureLibrary {
    textures: HashMap<TextureId, Arc<Texture>>,
}

impl TextureLibrary {
    // TODO: implement on the fly shader loading and unloading.
    pub fn load_as_needed() -> Self {
        todo!();
    }

    pub fn load_all() -> Self {
        todo!();
    }

    pub fn get(&self, id: TextureId) -> &Texture {
        &self
            .textures
            .get(&id)
            .expect("tried to access texture with bad id")
    }
}
