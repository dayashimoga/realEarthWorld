//! Off-screen framebuffer screenshot extraction and PPM serialization.

use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Captures a rendered wgpu texture and saves it to a binary PPM file on disk.
pub fn capture_texture_to_ppm(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let unpadded_bytes_per_row = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT; // 256
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
    let buffer_size = (padded_bytes_per_row * height) as u64;

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenshot Staging Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenshot Encoder"),
    });

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &staging_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });

    device.poll(wgpu::Maintain::Wait);
    rx.recv()??;

    {
        let data = buffer_slice.get_mapped_range();
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(output_path)?;
        // Write binary PPM header (P6, width, height, maxval 255)
        write!(file, "P6\n{} {}\n255\n", width, height)?;

        // Extract RGB channels, stripping alpha and padding
        let mut row_rgb = Vec::with_capacity((width * 3) as usize);
        for row in 0..height {
            row_rgb.clear();
            let row_start = (row * padded_bytes_per_row) as usize;
            for col in 0..width {
                let pixel_start = row_start + (col * 4) as usize;
                let r = data[pixel_start];
                let g = data[pixel_start + 1];
                let b = data[pixel_start + 2];
                row_rgb.push(r);
                row_rgb.push(g);
                row_rgb.push(b);
            }
            file.write_all(&row_rgb)?;
        }
    }

    staging_buffer.unmap();
    Ok(())
}
