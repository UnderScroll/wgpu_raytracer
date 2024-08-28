#![allow(dead_code)]

use crate::colors::Rgba;
use anyhow::{anyhow, Result};
use wgpu::util::DeviceExt;

pub struct Texture {
    pixels: Vec<Rgba<u8>>,
    pub width: usize,
    pub height: usize,
}

impl Texture {
    pub fn new(pixels: Vec<Rgba<u8>>, width: usize, height: usize) -> Self {
        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgba<u8>) -> Result<()> {
        if x >= self.width {
            return Err(anyhow!("Out of bound : {x} >= {0}", self.width));
        }
        if y >= self.height {
            return Err(anyhow!("Out of bound : {y} >= {0}", self.height));
        }

        self.pixels[y * self.width + x] = color;

        Ok(())
    }

    pub fn pixels(&self) -> &[Rgba<u8>] {
        self.pixels.as_slice()
    }

    pub fn data(&self) -> &[u8] {
        bytemuck::cast_slice(self.pixels.as_slice())
    }

    pub fn into_wgpu_texture(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        usage: wgpu::TextureUsages,
    ) -> wgpu::Texture {
        device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Test Texture"),
                size: wgpu::Extent3d {
                    width: self.width as u32,
                    height: self.height as u32,
                    depth_or_array_layers: 1,
                },
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                mip_level_count: 1,
                sample_count: 1,
                usage,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            self.data(),
        )
    }
}
