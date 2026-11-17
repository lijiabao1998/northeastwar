//! Shop boards. Heihe's are a line of Chinese over a line of Russian, in
//! white or gold on blue, red, green or purple. The names come from the map;
//! the lettering is set here with Noto Sans.

use bevy::{
    asset::RenderAssetUsages,
    mesh::Indices,
    prelude::*,
    render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
};
use sim::scene::Board;

// Low enough that a board is never squeezed to less than half its texture on screen: with no
// mip levels, anything finer turns into sparkle.
const PX_PER_METRE: f32 = 40.0;

/// A shop board on screen, so --bench can switch them off.
#[derive(Component)]
pub struct BoardMark;

struct Fonts {
    han: fontdue::Font,
    latin: fontdue::Font,
}

fn fonts() -> Option<Fonts> {
    let dir = sim::data_dir().parent()?.join("assets/fonts");
    let han = std::fs::read(dir.join("NotoSansSC-VF.ttf")).ok()?;
    let latin = std::fs::read(dir.join("NotoSans-Regular.ttf")).ok()?;
    Some(Fonts {
        han: fontdue::Font::from_bytes(han, fontdue::FontSettings::default()).ok()?,
        latin: fontdue::Font::from_bytes(latin, fontdue::FontSettings::default()).ok()?,
    })
}

/// One line of text, drawn into `canvas` centred on `centre_x`, baseline at `base_y`.
fn draw_line(canvas: &mut [u8], w: usize, h: usize, fonts: &Fonts, text: &str, px: f32, centre_x: f32, base_y: f32, ink: [u8; 3]) {
    let pick = |c: char| if fonts.latin.lookup_glyph_index(c) != 0 { &fonts.latin } else { &fonts.han };
    let mut total = 0.0;
    for c in text.chars() {
        total += pick(c).metrics(c, px).advance_width;
    }
    // Too long for the board: set it smaller rather than cut it off.
    let px = if total > w as f32 - 12.0 { px * (w as f32 - 12.0) / total } else { px };
    let mut pen = centre_x - text.chars().map(|c| pick(c).metrics(c, px).advance_width).sum::<f32>() * 0.5;
    for c in text.chars() {
        let (m, bitmap) = pick(c).rasterize(c, px);
        let x0 = (pen + m.xmin as f32).round() as i32;
        let y0 = (base_y - m.ymin as f32 - m.height as f32).round() as i32;
        for gy in 0..m.height {
            for gx in 0..m.width {
                let (x, y) = (x0 + gx as i32, y0 + gy as i32);
                if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                    continue;
                }
                let a = bitmap[gy * m.width + gx];
                let i = (y as usize * w + x as usize) * 4;
                if a > canvas[i + 3] {
                    canvas[i] = ink[0];
                    canvas[i + 1] = ink[1];
                    canvas[i + 2] = ink[2];
                    canvas[i + 3] = a;
                }
            }
        }
        pen += m.advance_width;
    }
}

pub fn spawn(
    commands: &mut Commands,
    boards: &[Board],
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    let Some(fonts) = fonts() else {
        warn!("no fonts found: shop boards will be blank");
        return;
    };
    for board in boards {
        let w = (board.width * PX_PER_METRE) as usize;
        let h = (board.height * PX_PER_METRE) as usize;
        let mut canvas = vec![0u8; w * h * 4];
        let ink = [(board.ink.r * 255.0) as u8, (board.ink.g * 255.0) as u8, (board.ink.b * 255.0) as u8];
        // Transparent pixels carry the ink colour too, so edges do not fringe dark.
        for px in canvas.chunks_mut(4) {
            px[0] = ink[0];
            px[1] = ink[1];
            px[2] = ink[2];
        }
        let hf = h as f32;
        match (board.zh.is_empty(), board.ru.is_empty()) {
            (false, false) => {
                draw_line(&mut canvas, w, h, &fonts, &board.zh, hf * 0.52, w as f32 * 0.5, hf * 0.56, ink);
                draw_line(&mut canvas, w, h, &fonts, &board.ru.to_uppercase(), hf * 0.26, w as f32 * 0.5, hf * 0.9, ink);
            }
            (false, true) => draw_line(&mut canvas, w, h, &fonts, &board.zh, hf * 0.7, w as f32 * 0.5, hf * 0.78, ink),
            (true, false) => draw_line(&mut canvas, w, h, &fonts, &board.ru.to_uppercase(), hf * 0.5, w as f32 * 0.5, hf * 0.7, ink),
            _ => continue,
        }
        // Smaller copies of the board for when it is far away. Without them the
        // strokes of the letters break up into sparkle.
        let mut data = canvas.clone();
        let (mut lw, mut lh, mut level, mut levels) = (w, h, canvas, 1u32);
        while lw > 1 || lh > 1 {
            let (nw, nh) = ((lw / 2).max(1), (lh / 2).max(1));
            let mut next = vec![0u8; nw * nh * 4];
            for y in 0..nh {
                for x in 0..nw {
                    let mut alpha = 0u32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (sx, sy) = ((x * 2 + dx).min(lw - 1), (y * 2 + dy).min(lh - 1));
                        alpha += level[(sy * lw + sx) * 4 + 3] as u32;
                    }
                    let o = (y * nw + x) * 4;
                    next[o] = ink[0];
                    next[o + 1] = ink[1];
                    next[o + 2] = ink[2];
                    next[o + 3] = (alpha / 4) as u8;
                }
            }
            data.extend_from_slice(&next);
            level = next;
            lw = nw;
            lh = nh;
            levels += 1;
        }
        let mut image = Image::new(
            Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 },
            TextureDimension::D2,
            vec![0u8; w * h * 4],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.mip_level_count = levels;
        image.data = Some(data);
        image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
            mag_filter: bevy::image::ImageFilterMode::Linear,
            min_filter: bevy::image::ImageFilterMode::Linear,
            mipmap_filter: bevy::image::ImageFilterMode::Linear,
            ..default()
        });
        let texture = images.add(image);
        let c = Vec3::from(board.centre) + Vec3::from(board.normal) * 0.02;
        let r = Vec3::from(board.right) * (board.width * 0.5);
        let u = Vec3::Y * (board.height * 0.5);
        let n: [f32; 3] = board.normal;
        let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![(c - r - u).to_array(), (c + r - u).to_array(), (c + r + u).to_array(), (c - r + u).to_array()])
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![n, n, n, n])
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]])
            .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
        commands.spawn((
            BoardMark,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(texture.clone()),
                // A lightbox: the letters carry their own light, day and night.
                emissive_texture: Some(texture),
                emissive: LinearRgba::new(30.0, 30.0, 30.0, 1.0),
                emissive_exposure_weight: 1.0,
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 0.6,
                ..default()
            })),
        ));
    }
}
