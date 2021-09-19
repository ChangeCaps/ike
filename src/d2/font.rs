use std::{collections::HashMap, io::Read, path::Path};

use ab_glyph::{Font as FontTrait, FontArc, PxScale, ScaleFont};
use glam::{UVec2, Vec2};

use crate::prelude::{Color, Color8, Texture};

#[derive(Clone, Debug, PartialEq)]
pub struct RawGlyph {
    pub min: UVec2,
    pub max: UVec2,
    pub size: Vec2,
    pub line_height: f32,
    pub left_bearing: f32,
    pub right_bearing: f32,
}

impl RawGlyph {
    #[inline]
    pub fn width(&self) -> u32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.max.y - self.min.y
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub min: Vec2,
    pub max: Vec2,
}

impl Glyph {
    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

pub struct Font {
    pub texture: Texture,
    pub glyphs: HashMap<char, RawGlyph>,
}

impl Font {
    #[inline]
    pub fn load(path: impl AsRef<Path>, scale: impl Into<PxScale>) -> anyhow::Result<Self> {
        let mut bytes = Vec::new();
        std::fs::File::open(path.as_ref())?.read_to_end(&mut bytes)?;
        let font = FontArc::try_from_vec(bytes)?;
        let font = font.as_scaled(scale);

        let mut width = 1;
        let mut row_height = 1;

        for (_glyph, c) in font.codepoint_ids() {
            let glyph = font.scaled_glyph(c);

            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();

                width += bounds.width() as usize;
                row_height = row_height.max(bounds.height() as usize);
            }
        }

        let area = width * row_height;
        width = (area as f32).sqrt().round() as usize;

        row_height += 2;

        let mut x = 0;
        let mut y = 0;
        let mut height = row_height;

        let mut data: Vec<Color8> = Vec::new();

        let mut glyphs = HashMap::new();

        for (_, c) in font.codepoint_ids() {
            let glyph = font.scaled_glyph(c);

            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();

                let mut x_end = x + bounds.width() as usize;

                if x_end >= width {
                    x = 0;
                    x_end = bounds.width() as usize;

                    y += row_height;
                    height += row_height;
                }

                let y_bound = bounds.min.y / row_height as f32;

                let advance = font.h_advance(outlined.glyph().id) / row_height as f32;
                let side_bearing = font.h_side_bearing(outlined.glyph().id) / row_height as f32;

                glyphs.insert(
                    c,
                    RawGlyph {
                        min: UVec2::new(x as u32, y as u32),
                        max: UVec2::new(x_end as u32, y as u32 + bounds.height() as u32),
                        size: Vec2::new(bounds.width(), bounds.height()) / row_height as f32,
                        line_height: -y_bound / 2.0,
                        left_bearing: side_bearing,
                        right_bearing: advance - bounds.width() / row_height as f32 - side_bearing,
                    },
                );

                let data_size = (height + 1) * width;
                data.resize(data_size, Color8::TRANSPARENT);

                outlined.draw(|dx, dy, a| {
                    let x = x + dx as usize;
                    let y = y + dy as usize;
                    data[y * width + x] = Color::rgba(1.0, 1.0, 1.0, a).into();
                });

                x = x_end + 2;
            }
        }

        Ok(Self {
            texture: Texture::from_data(data, width as u32, height as u32),
            glyphs,
        })
    }

    #[inline]
    pub fn raw_glyph(&self, c: char) -> Option<&RawGlyph> {
        self.glyphs.get(&c)
    }

    #[inline]
    pub fn glyph(&self, c: char) -> Option<Glyph> {
        let glyph = self.glyphs.get(&c)?;

        Some(Glyph {
            min: glyph.min.as_f32() / self.texture.size().as_f32(),
            max: glyph.max.as_f32() / self.texture.size().as_f32(),
        })
    }
}
