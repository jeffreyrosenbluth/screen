use crate::core::{App, BlendMode, Combine, ImgGrid, LineColor, SortBy, SortKey, SortOrder};
use crate::matrix::Matrix;
use crate::sortfns::*;
use fastrand;
use image::*;
use palette::{blend::Blend, LinSrgba, Srgba};
use rayon::prelude::*;
use std::sync::Arc;
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

    match app.combine {
        Combine::Warp => {
            println!("Warping Image");
            let w = app.width as f32;
            let h = app.height as f32;
            let img_noise =
                ImgNoise::new(DynamicImage::ImageRgba8(blurred_img_2)).set_map(ColorMap::Lightness);
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
        }
        Combine::Unsort => {
            println!("Unsorting Image");
            let img_1 = DynamicImage::ImageRgba8(blurred_img_1);
            let img_2 = DynamicImage::ImageRgba8(blurred_img_2);
            let sort_fn = match app.sort_key {
                SortKey::Lightness => luma,
                SortKey::Hue => hue,
                SortKey::Saturation => sat,
            };
            let px_map = match app.sort_by {
                SortBy::Row => pixel_map_row(&img_1, sort_fn, app.row_sort_order, None),
                SortBy::Column => pixel_map_column(&img_1, sort_fn, app.col_sort_order, None),
                SortBy::RowCol => {
                    let pm = pixel_map_row(&img_1, sort_fn, app.row_sort_order, None);
                    pixel_map_column(&img_1, sort_fn, app.col_sort_order, Some(pm))
                }
                SortBy::ColRow => {
                    let pm = pixel_map_column(&img_1, sort_fn, app.col_sort_order, None);
                    pixel_map_row(&img_1, sort_fn, app.row_sort_order, Some(pm))
                }
            };
            img = pixel_unsort(&img_2, &px_map);
        }
        rest @ (Combine::Blend | Combine::Divide | Combine::Mix) => {
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

            println!("Generating Image");
            img.par_enumerate_pixels_mut().for_each(|(x, y, px)| {
                let pixel;
                match rest {
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
                    _ => unreachable!(),
                }
                px[0] = pixel[0];
                px[1] = pixel[1];
                px[2] = pixel[2];
                px[3] = pixel[3];
            });
        }
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

    if app.grain_scale > 0.0 && app.grain_factor > 0.0 {
        let gr = Grain::new(500, 500, app.grain_scale, app.grain_factor);
        gr.canvas_grain(&mut canvas);
    }

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

// Generate an image grid with the location of each pixel in the image.
// Sort the pixels in each row by the sort function.
pub fn pixel_map_row(
    img: &DynamicImage,
    f: SortFn,
    order: SortOrder,
    grid: Option<ImgGrid>,
) -> ImgGrid {
    let mut px_map = match grid {
        Some(g) => g,
        None => Matrix::generate(img.width() as usize, img.height() as usize, |x, y| (x, y)),
    };
    for y in 0..px_map.height {
        let mut row = px_map[y].to_vec();
        row.par_sort_by_key(|x| order.dir() * f(img.get_pixel(x.0 as u32, x.1 as u32)));
        let mut indices = (0..row.len()).collect::<Vec<_>>();
        indices.par_sort_by_key(|i| row[*i].0);
        let row1 = indices.par_iter().map(|i| (*i, y)).collect::<Vec<_>>();
        px_map[y].par_iter_mut().enumerate().for_each(|(i, e)| {
            *e = row1[i];
        });
    }
    px_map
}

// Generate an image grid with the location of each pixel in the image.
// Sort the pixels in each column by the sort function.
pub fn pixel_map_column(
    img: &DynamicImage,
    f: SortFn,
    order: SortOrder,
    grid: Option<ImgGrid>,
) -> ImgGrid {
    let mut px_map = match grid {
        Some(g) => g,
        None => Matrix::generate(img.width() as usize, img.height() as usize, |x, y| (x, y)),
    };
    for x in 0..px_map.width {
        let mut column = px_map.get_column(x);
        column.par_sort_by_key(|y| order.dir() * f(img.get_pixel(y.0 as u32, y.1 as u32)));
        let mut indices = (0..column.len()).collect::<Vec<_>>();
        indices.par_sort_by_key(|i| column[*i].1);
        let column1 = indices.par_iter().map(|i| (x, *i)).collect::<Vec<_>>();
        for y in 0..px_map.height {
            px_map[y][x] = column1[y]
        }
    }
    px_map
}

#[allow(dead_code)]
// Pixel sort a DynamicImage by rows.
pub fn pixel_sort_row(img: &DynamicImage, f: SortFn, order: SortOrder) -> RgbaImage {
    let mut data: Vec<u8> = Vec::with_capacity(16 * img.width() as usize * img.height() as usize);
    let buffer = img.to_rgba8();
    for buf_row in buffer.rows() {
        let mut row = Vec::with_capacity(buf_row.len());
        for p in buf_row {
            row.push(*p);
        }
        row.par_sort_by_key(|p| order.dir() * f(*p));
        for p in row {
            for c in p.channels() {
                data.push(*c);
            }
        }
    }
    ImageBuffer::from_vec(img.width(), img.height(), data).unwrap()
}

#[allow(dead_code)]
// Pixel sort a DynamicImage by columns.
pub fn pixel_sort_column(img: &DynamicImage, f: SortFn, order: SortOrder) -> RgbaImage {
    let rotate_img = img.rotate90();
    let sorted_img = pixel_sort_row(&rotate_img, f, -order);
    let dyn_img = DynamicImage::ImageRgba8(sorted_img);
    dyn_img.rotate270().into_rgba8()
}

// Unsort the image using the pixel map.
pub fn pixel_unsort(img: &DynamicImage, px_map: &ImgGrid) -> RgbaImage {
    let mut out_image = RgbaImage::new(img.width(), img.height());
    for y in 0..px_map.height {
        for x in 0..px_map.width {
            let (x1, y1) = px_map[y][x];
            let p = img.get_pixel(x1 as u32, y1 as u32);
            out_image.put_pixel(x as u32, y as u32, p)
        }
    }
    out_image
}
