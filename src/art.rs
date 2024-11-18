use std::sync::Arc;

use crate::core::{App, BlendMode, Combine, LineColor, MapColor};
use fastrand;
use image::{DynamicImage, RgbaImage};
use palette::{blend::Blend, LinSrgba, Srgba};
use rayon::prelude::*;
use wassily::prelude::*;

pub(crate) fn draw(app: &App) -> RgbaImage {
    println!("\n-------- Screen 0.1 ---------");
    fastrand::seed(13);
    println!("Resizing Image 1 to {}x{}", app.width, app.height);
    let img_1 = DynamicImage::ImageRgba8(app.img_1.clone())
        .huerotate(app.hue_rotation_1)
        .resize_exact(app.width, app.height, image::imageops::FilterType::Lanczos3);

    println!("Resizing Image 2 to {}x{}", app.width, app.height);
    let img_2 = DynamicImage::ImageRgba8(app.img_2.clone())
        .huerotate(app.hue_rotation_2)
        .resize_exact(app.width, app.height, image::imageops::FilterType::Lanczos3);

    println!("Blurring Image 1");
    let mut img = RgbaImage::new(app.width, app.height);

    let blurred_img_1 = if app.img_blur_1 > 0.0 {
        img_1.fast_blur(app.img_blur_1).to_rgba8()
    } else {
        img_1.to_rgba8()
    };

    println!("Blurring Image 2");
    let blurred_img_2 = if app.img_blur_2 > 0.0 {
        img_2.fast_blur(app.img_blur_2).to_rgba8()
    } else {
        img_2.to_rgba8()
    };

    println!("Generating Image");
    if app.combine == Combine::Warp {
        let w = app.width as f32;
        let h = app.height as f32;
        let cm = match app.color_map {
            MapColor::RedGreen => ColorMap::RedGreen,
            MapColor::YellowBlue => ColorMap::YellowBlue,
            _ => ColorMap::Lightness,
        };
        let img_noise = ImgNoise::new(DynamicImage::ImageRgba8(blurred_img_2)).set_map(cm);
        let angle_opts = NoiseOpts::default()
            .scales(app.angle_scale)
            .factor(app.angle_factor)
            .width(w)
            .height(h);
        let radius_opts = NoiseOpts::default()
            .scales(app.radius_scale)
            .factor(app.radius_factor)
            .width(w)
            .height(h);
        let img_1 = DynamicImage::ImageRgba8(blurred_img_1);
        let warp = Warp::with_image(
            Arc::new(move |z| {
                pt(
                    noise2d(&img_noise, &angle_opts, z.x, z.y),
                    noise2d_01(&img_noise, &radius_opts, z.x + w / 2.9887, z.y + h / 2.9973),
                )
            }),
            &img_1,
            w,
            h,
            Coord::Polar,
        );
        img.par_enumerate_pixels_mut().for_each(|(x, y, px)| {
            let pixel = warp.get_reflected(x as f32, y as f32);
            px[0] = (pixel.red() * 255.0) as u8;
            px[1] = (pixel.green() * 255.0) as u8;
            px[2] = (pixel.blue() * 255.0) as u8;
            px[3] = (pixel.alpha() * 255.0) as u8;
        });
    } else {
        let opts = NoiseOpts::default()
            .scales(5.0)
            .width(app.width as f32)
            .height(app.height as f32);

        let nf = Fbm::<Perlin>::default()
            .set_seed(13)
            .set_octaves(app.octaves);

        let opts2 = NoiseOpts::default()
            .scales(5.0)
            .width(app.width as f32)
            .height(app.height as f32);

        let nf2 = Fbm::<Perlin>::default().set_seed(23).set_octaves(4);

        img.par_enumerate_pixels_mut().for_each(|(x, y, px)| {
            let pixel;
            match app.combine {
                Combine::Divide => {
                    if noise2d(&nf, &opts, x as f32, y as f32)
                        + noise2d(&nf2, &opts2, x as f32, y as f32)
                            * app.contamination
                            * (0.5 - fastrand::f32())
                            / (1.0 + 0.5 * app.contamination)
                        > app.cutoff
                    {
                        pixel = *blurred_img_1.get_pixel(x, y);
                    } else {
                        pixel = *blurred_img_2.get_pixel(x, y);
                    }
                }
                Combine::Blend => {
                    pixel = blend(
                        *blurred_img_1.get_pixel(x, y),
                        *blurred_img_2.get_pixel(x, y),
                        app.mode,
                    );
                }
                Combine::Mix => {
                    if noise2d(&nf, &opts, x as f32, y as f32)
                        + noise2d(&nf2, &opts2, x as f32, y as f32)
                            * app.contamination
                            * (0.5 - fastrand::f32())
                            / (1.0 + 0.5 * app.contamination)
                        > app.cutoff
                    {
                        pixel = blend(
                            *blurred_img_1.get_pixel(x, y),
                            *blurred_img_2.get_pixel(x, y),
                            app.mode,
                        )
                    } else {
                        pixel = blend(
                            *blurred_img_2.get_pixel(x, y),
                            *blurred_img_1.get_pixel(x, y),
                            app.mode,
                        )
                    };
                }
                Combine::Warp => unreachable!(),
            }
            px[0] = pixel[0];
            px[1] = pixel[1];
            px[2] = pixel[2];
            px[3] = pixel[3];
        });
    }

    println!("Creating Canvas");
    let mut canvas = Canvas::from_image(&DynamicImage::ImageRgba8(img));
    let linecolor = if app.line_color == LineColor::Black {
        *BLACK
    } else {
        *WHITE
    };
    let mut i = app.spacing;

    println!("Drawing Overlay");
    if app.screen {
        while i < canvas.h_f32() {
            let v0 = pt(0, i);
            let v1 = pt(canvas.width(), i);
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
        while i < canvas.w_f32() {
            let v0 = pt(i, 0);
            let v1 = pt(i, canvas.height());
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
    println!("Image Generated");
    println!("-----------------------------");

    canvas_to_rgba_image(&canvas)
}

fn canvas_to_rgba_image(canvas: &Canvas) -> RgbaImage {
    let pixmap = &canvas.pixmap;
    let pixels = pixmap.data();
    // Note: tiny-skia uses premultiplied alpha, so we need to unpremultiply
    ImageBuffer::from_fn(pixmap.width(), pixmap.height(), |x, y| {
        let idx = (y * pixmap.width() + x) as usize * 4;
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

fn srgba_to_rgba_u8(color: Srgba<f32>) -> Rgba<u8> {
    let rgba = color.into_format::<u8, u8>().into_components();
    Rgba([rgba.0, rgba.1, rgba.2, rgba.3])
}

fn blend(c1: Rgba<u8>, c2: Rgba<u8>, mode: BlendMode) -> Rgba<u8> {
    let c1 = Srgba::from_components((c1.0[0], c1.0[1], c1.0[2], c1.0[3]));
    let c2 = Srgba::from_components((c2.0[0], c2.0[1], c2.0[2], c2.0[3]));
    let lin_color1: LinSrgba = c1.into_linear();
    let lin_color2: LinSrgba = c2.into_linear();
    let blended_lin_color = match mode {
        BlendMode::Multiply => lin_color1.multiply(lin_color2),
        BlendMode::Screen => lin_color1.screen(lin_color2),
        BlendMode::Overlay => lin_color1.overlay(lin_color2),
        BlendMode::Darken => lin_color1.darken(lin_color2),
        BlendMode::Lighten => Blend::lighten(lin_color1, lin_color2),
        BlendMode::Dodge => lin_color1.dodge(lin_color2),
        BlendMode::Burn => lin_color1.burn(lin_color2),
        BlendMode::HardLight => lin_color1.hard_light(lin_color2),
        BlendMode::SoftLight => lin_color1.soft_light(lin_color2),
        BlendMode::Difference => lin_color1.difference(lin_color2),
        BlendMode::Exclusion => lin_color1.exclusion(lin_color2),
    };
    let blended_srgba = Srgba::from_linear(blended_lin_color);
    srgba_to_rgba_u8(blended_srgba)
}
