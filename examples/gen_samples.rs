fn main() {
    use image::{Rgb, RgbImage};
    std::fs::create_dir_all("samples").ok();
    let mut img = RgbImage::new(256, 256);
    for y in 0..256u32 {
        for x in 0..256u32 {
            let dx = x as f32 - 128.0;
            let dy = y as f32 - 128.0;
            let d = (dx*dx + dy*dy).sqrt();
            let t = (1.0 - (d / 120.0).clamp(0.0, 1.0)).powf(1.4);
            let r = (40.0 + 200.0 * t) as u8;
            let g = (30.0 + 90.0 * t) as u8;
            let b = (80.0 + 175.0 * t) as u8;
            img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }
    img.save("samples/orb.png").unwrap();
    let mut g = RgbImage::new(320, 180);
    for y in 0..180u32 {
        for x in 0..320u32 {
            let t = x as f32 / 319.0;
            g.put_pixel(x, y, Rgb([(t*255.0) as u8, 40, ((1.0-t)*255.0) as u8]));
        }
    }
    g.save("samples/gradient.png").unwrap();
    println!("ok");
}
