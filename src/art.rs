use crate::core::App;
use fastrand;
use image::{DynamicImage, RgbaImage};
use rayon::prelude::*;
use wassily::prelude::*;

pub(crate) fn draw(app: &App) -> RgbaImage {
    let img_1 = DynamicImage::ImageRgba8(app.img_1.clone()).resize_exact(
        app.width,
        app.height,
        image::imageops::FilterType::Lanczos3,
    );
    let img_2 = DynamicImage::ImageRgba8(app.img_2.clone()).resize_exact(
        app.width,
        app.height,
        image::imageops::FilterType::Lanczos3,
    );
    let mut img = RgbaImage::new(app.width, app.height);
    let blurred_img_1 = img_1.fast_blur(app.img_blur_1).to_rgba8();
    let blurred_img_2 = img_2.fast_blur(app.img_blur_2).to_rgba8();

    let opts = NoiseOpts::default()
        .scales(5.0)
        .width(app.width as f32)
        .height(app.height as f32);

    let nf = Fbm::<Perlin>::default()
        .set_seed(13)
        .set_octaves(app.octaves);

    let mut samples = img.as_flat_samples_mut();
    let raw_buf = samples.as_mut_slice();
    raw_buf
        .par_chunks_exact_mut(4) // Each pixel is 4 bytes (RGBA)
        .enumerate()
        .for_each(|(i, pixel_bytes)| {
            let x = (i as u32) % app.width;
            let y = (i as u32) / app.width;
            let pixel;
            if (noise2d(&nf, &opts, x as f32, y as f32)
                + app.contamination * (0.5 - fastrand::f32()))
                / (1.0 + 0.5 * app.contamination)
                > app.cutoff
            {
                pixel = *blurred_img_1.get_pixel(x, y);
            } else {
                pixel = *blurred_img_2.get_pixel(x, y);
            }
            pixel_bytes[0] = pixel[0]; // R
            pixel_bytes[1] = pixel[1]; // G
            pixel_bytes[2] = pixel[2]; // B
            pixel_bytes[3] = pixel[3]; // A
        });

    let mut canvas = Canvas::from_image(&DynamicImage::ImageRgba8(img));

    let width = canvas.width();
    let height = canvas.height();
    let linecolor = *BLACK;
    let w = canvas.w_f32();
    let h = canvas.h_f32();
    let mut i = app.spacing;

    if app.screen {
        while i < h {
            let v0 = pt(0, i);
            let v1 = pt(width, i);
            let mut fl = FadeLine::new(v0, v1, 98731 + i as u64)
                .subdivisions(app.subdivisions)
                .thickness(app.thickness)
                .min_opacity(app.min_opacity)
                .max_opacity(app.max_opacity)
                .color(linecolor);
            fl.draw(&mut canvas);
            i += app.spacing;
        }
        i = app.spacing;
        while i < w {
            let v0 = pt(i, 0);
            let v1 = pt(i, height);
            let mut fl = FadeLine::new(v0, v1, 98731 + i as u64)
                .subdivisions(app.subdivisions)
                .thickness(app.thickness)
                .min_opacity(app.min_opacity)
                .max_opacity(app.max_opacity)
                .color(linecolor);
            fl.draw(&mut canvas);
            i += app.spacing;
        }
    }
    canvas_to_rgba_image(&canvas)
}

fn canvas_to_rgba_image(canvas: &Canvas) -> RgbaImage {
    let pixmap = &canvas.pixmap;
    let width = pixmap.width();
    let height = pixmap.height();
    let pixels = pixmap.data();

    // Create a new RgbaImage
    // Note: tiny-skia uses premultiplied alpha, so we need to unpremultiply
    ImageBuffer::from_fn(width, height, |x, y| {
        let idx = (y * width + x) as usize * 4;
        let pixel = &pixels[idx..idx + 4];

        // Unpremultiply alpha
        let a = pixel[3];
        let (r, g, b) = if a > 0 {
            (
                ((pixel[0] as u32 * 255) / (a as u32)) as u8,
                ((pixel[1] as u32 * 255) / (a as u32)) as u8,
                ((pixel[2] as u32 * 255) / (a as u32)) as u8,
            )
        } else {
            (0, 0, 0)
        };

        image::Rgba([r, g, b, a])
    })
}
