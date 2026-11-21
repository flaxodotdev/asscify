//! Image → ASCII art conversion with multiple styles and optional color.

use image::{DynamicImage, GenericImageView, Rgba};
use ratatui::style::Color;
use ratatui::text::{Line, Span};

/// Character density ramps — darker glyphs for denser pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AsciiStyle {
    #[default]
    Classic,
    Blocks,
    Detailed,
    Minimal,
    Binary,
}

impl AsciiStyle {
    pub fn name(self) -> &'static str {
        match self {
            AsciiStyle::Classic => "classic",
            AsciiStyle::Blocks => "blocks",
            AsciiStyle::Detailed => "detailed",
            AsciiStyle::Minimal => "minimal",
            AsciiStyle::Binary => "binary",
        }
    }

    pub fn ramp(self) -> &'static str {
        match self {
            AsciiStyle::Classic => " .:-=+*#%@",
            AsciiStyle::Blocks => " ░▒▓█",
            AsciiStyle::Detailed => {
                " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"
            }
            AsciiStyle::Minimal => " .oO@",
            AsciiStyle::Binary => " 01",
        }
    }

    pub fn next(self) -> Self {
        match self {
            AsciiStyle::Classic => AsciiStyle::Blocks,
            AsciiStyle::Blocks => AsciiStyle::Detailed,
            AsciiStyle::Detailed => AsciiStyle::Minimal,
            AsciiStyle::Minimal => AsciiStyle::Binary,
            AsciiStyle::Binary => AsciiStyle::Classic,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            AsciiStyle::Classic => AsciiStyle::Binary,
            AsciiStyle::Blocks => AsciiStyle::Classic,
            AsciiStyle::Detailed => AsciiStyle::Blocks,
            AsciiStyle::Minimal => AsciiStyle::Detailed,
            AsciiStyle::Binary => AsciiStyle::Minimal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsciiOptions {
    pub width: u32,
    pub style: AsciiStyle,
    pub invert: bool,
    pub color: bool,
    /// Aspect correction — terminal cells are taller than wide.
    pub aspect: f32,
}

impl Default for AsciiOptions {
    fn default() -> Self {
        Self {
            width: 80,
            style: AsciiStyle::Classic,
            invert: false,
            color: true,
            aspect: 0.45,
        }
    }
}

/// One cell of colored ASCII.
#[derive(Debug, Clone)]
pub struct AsciiCell {
    pub ch: char,
    pub color: Color,
}

/// A full ASCII frame ready for terminal rendering or plain export.
#[derive(Debug, Clone)]
pub struct AsciiArt {
    pub width: usize,
    pub height: usize,
    pub rows: Vec<Vec<AsciiCell>>,
}

impl AsciiArt {
    pub fn as_plain_string(&self) -> String {
        let mut out = String::with_capacity(self.height * (self.width + 1));
        for row in &self.rows {
            for cell in row {
                out.push(cell.ch);
            }
            out.push('\n');
        }
        out
    }

    /// Compact lines that reuse style runs for faster rendering.
    pub fn as_compact_lines(&self) -> Vec<Line<'static>> {
        self.rows
            .iter()
            .map(|row| {
                let mut spans = Vec::new();
                let mut buf = String::new();
                let mut current: Option<Color> = None;

                for cell in row {
                    match current {
                        Some(c) if c == cell.color => buf.push(cell.ch),
                        Some(_) => {
                            spans.push(Span::styled(
                                std::mem::take(&mut buf),
                                ratatui::style::Style::default().fg(current.unwrap()),
                            ));
                            buf.push(cell.ch);
                            current = Some(cell.color);
                        }
                        None => {
                            buf.push(cell.ch);
                            current = Some(cell.color);
                        }
                    }
                }
                if !buf.is_empty() {
                    spans.push(Span::styled(
                        buf,
                        ratatui::style::Style::default().fg(current.unwrap_or(Color::White)),
                    ));
                }
                Line::from(spans)
            })
            .collect()
    }
}

fn luminance(px: Rgba<u8>) -> f32 {
    let a = px[3] as f32 / 255.0;
    let r = px[0] as f32 / 255.0;
    let g = px[1] as f32 / 255.0;
    let b = px[2] as f32 / 255.0;
    // Rec. 709, premultiplied by alpha so transparent = dark/empty.
    (0.2126 * r + 0.7152 * g + 0.0722 * b) * a
}

fn to_ratatui_color(px: Rgba<u8>) -> Color {
    if px[3] < 16 {
        return Color::DarkGray;
    }
    Color::Rgb(px[0], px[1], px[2])
}

/// Convert an image into ASCII art sized for the given options.
pub fn image_to_ascii(img: &DynamicImage, opts: &AsciiOptions) -> AsciiArt {
    let width = opts.width.max(8);
    let (iw, ih) = img.dimensions();
    if iw == 0 || ih == 0 {
        return AsciiArt {
            width: 0,
            height: 0,
            rows: vec![],
        };
    }

    let aspect = opts.aspect.clamp(0.2, 1.0);
    let height = ((ih as f32 / iw as f32) * width as f32 * aspect).round().max(1.0) as u32;

    let resized = img.resize_exact(width, height, image::imageops::FilterType::Triangle);
    let ramp: Vec<char> = opts.style.ramp().chars().collect();
    let ramp_len = ramp.len() as f32;

    let mut rows = Vec::with_capacity(height as usize);

    for y in 0..height {
        let mut row = Vec::with_capacity(width as usize);
        for x in 0..width {
            let px = resized.get_pixel(x, y);
            // Terminal-first: brighter pixels → denser glyphs so color shows on dark BGs.
            let mut density = luminance(px);
            if opts.invert {
                density = 1.0 - density;
            }

            let idx = (density * (ramp_len - 1.0)).round() as usize;
            let idx = idx.min(ramp.len() - 1);
            let ch = ramp[idx];

            let color = if opts.color {
                to_ratatui_color(px)
            } else {
                Color::White
            };

            row.push(AsciiCell { ch, color });
        }
        rows.push(row);
    }

    AsciiArt {
        width: width as usize,
        height: height as usize,
        rows,
    }
}

/// Load an image from disk and convert it.
pub fn convert_path(path: &std::path::Path, opts: &AsciiOptions) -> anyhow::Result<AsciiArt> {
    let img = image::open(path)?;
    Ok(image_to_ascii(&img, opts))
}

/// Built-in procedural showcase — a glowing gradient orb with rings.
pub fn showcase_art(width: u32, style: AsciiStyle, color: bool) -> AsciiArt {
    let width = width.max(20);
    let height = ((width as f32) * 0.45).round().max(10.0) as u32;
    let ramp: Vec<char> = style.ramp().chars().collect();
    let ramp_len = ramp.len() as f32;

    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;

    let mut rows = Vec::with_capacity(height as usize);
    for y in 0..height {
        let mut row = Vec::with_capacity(width as usize);
        for x in 0..width {
            // Correct for cell aspect so the orb looks round.
            let dx = (x as f32 - cx) / cx;
            let dy = (y as f32 - cy) / cy;
            let dist = (dx * dx + dy * dy).sqrt();

            let ring = ((dist * 8.0).sin().abs()) * 0.35;
            let core = (1.0 - (dist / 1.15).clamp(0.0, 1.0)).powf(1.6);
            let glow = (core + ring * core).clamp(0.0, 1.0);

            let idx = (glow * (ramp_len - 1.0)).round() as usize;
            let idx = idx.min(ramp.len() - 1);
            let ch = ramp[idx];

            let color = if color {
                // Cool silver / ice glow for dark terminals.
                let t = glow;
                let r = (30.0 + 200.0 * t + 20.0 * ring) as u8;
                let g = (34.0 + 210.0 * t) as u8;
                let b = (40.0 + 220.0 * t) as u8;
                Color::Rgb(r.min(255), g.min(255), b.min(255))
            } else {
                Color::White
            };

            row.push(AsciiCell { ch, color });
        }
        rows.push(row);
    }

    AsciiArt {
        width: width as usize,
        height: height as usize,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, RgbaImage};

    #[test]
    fn converts_solid_image() {
        let img: RgbaImage = ImageBuffer::from_pixel(10, 10, image::Rgba([255, 255, 255, 255]));
        let art = image_to_ascii(&DynamicImage::ImageRgba8(img), &AsciiOptions::default());
        assert!(art.width > 0);
        assert!(art.height > 0);
        assert!(!art.as_plain_string().is_empty());
    }

    #[test]
    fn style_cycle() {
        assert_eq!(AsciiStyle::Classic.next(), AsciiStyle::Blocks);
        assert_eq!(AsciiStyle::Classic.prev(), AsciiStyle::Binary);
    }
}
