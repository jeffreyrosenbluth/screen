use egui::{ColorImage, TextureHandle};
use image::{
    imageops::{self, FilterType},
    RgbaImage,
};
use std::sync::mpsc::Receiver;

use crate::matrix::Matrix;
use serde::{Deserialize, Serialize};
use std::ops::Neg;

pub fn dims(width: f32, height: f32) -> (f32, f32) {
    if width.max(height) <= 1200.0 {
        return (width, height);
    }
    let aspect_ratio = height / width;
    if width >= height {
        (1200.0, (1200.0 * aspect_ratio))
    } else {
        ((1200.0 / aspect_ratio), 1200.0)
    }
}

pub fn to_color_image(img: &RgbaImage, width: u32, height: u32) -> ColorImage {
    let img = imageops::resize(img, width, height, FilterType::Lanczos3);
    ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        &img.into_vec(),
    )
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Combine {
    Blend,
    Divide,
    Mix,
    Warp,
    Unsort,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum LineColor {
    Black,
    White,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    Dodge,
    Burn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub enum SortBy {
    Row,
    Column,
    ColRow,
    RowCol,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub enum SortKey {
    Lightness,
    Hue,
    Saturation,
}

// Used to store the location of each pixel in the sort image.
pub type ImgGrid = Matrix<(usize, usize)>;

// Sort by increasing or decreasing direction of the sort function.
#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn dir(self) -> i16 {
        match self {
            SortOrder::Ascending => 1,
            SortOrder::Descending => -1,
        }
    }
}

// Change the sort order with the unary negation operator.
impl Neg for SortOrder {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    pub img_path_1: Option<String>,
    pub img_path_2: Option<String>,
    pub img_blur_1: f32,
    pub img_blur_2: f32,
    pub hue_rotation_1: i32,
    pub hue_rotation_2: i32,
    pub opacity_1: u8,
    pub opacity_2: u8,
    pub width: u32,
    pub height: u32,
    pub spacing: f32,
    pub line_color: LineColor,
    pub thickness: f32,
    pub subdivisions: u32,
    pub min_opacity: f32,
    pub max_opacity: f32,
    pub contamination: f32,
    pub octaves: usize,
    pub cutoff: f32,
    pub mode: BlendMode,
    pub combine: Combine,
    pub screen: bool,
    pub angle_scale: f32,
    pub angle_factor: f32,
    pub radius_scale: f32,
    pub radius_factor: f32,
    pub sort_key: SortKey,
    pub sort_by: SortBy,
    pub row_sort_order: SortOrder,
    pub col_sort_order: SortOrder,
    pub grain_scale: f32,
    pub grain_factor: f32,

    #[serde(skip)]
    pub texture: Option<TextureHandle>,

    #[serde(skip)]
    pub img_1: RgbaImage,

    #[serde(skip)]
    pub img_2: RgbaImage,

    #[serde(skip)]
    pub img: RgbaImage,

    #[serde(skip)]
    pub drawing_in_progress: bool,

    #[serde(skip)]
    pub draw_receiver: Option<Receiver<RgbaImage>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            img_path_1: None,
            img_path_2: None,
            img_blur_1: 0.0,
            img_blur_2: 0.0,
            hue_rotation_1: 0,
            hue_rotation_2: 0,
            width: 4032,
            height: 3024,
            spacing: 25.0,
            line_color: LineColor::Black,
            thickness: 0.5,
            subdivisions: 50,
            min_opacity: 0.1,
            max_opacity: 0.9,
            contamination: 0.25,
            octaves: 2,
            cutoff: 0.0,
            mode: BlendMode::Screen,
            combine: Combine::Blend,
            screen: true,
            angle_scale: 5.0,
            angle_factor: 6.0,
            radius_scale: 5.0,
            radius_factor: 1000.0,
            sort_key: SortKey::Lightness,
            sort_by: SortBy::Row,
            row_sort_order: SortOrder::Ascending,
            col_sort_order: SortOrder::Ascending,
            grain_factor: 10.0,
            grain_scale: 0.35,
            texture: None,
            img_1: RgbaImage::new(1, 1),
            img_2: RgbaImage::new(1, 1),
            img: RgbaImage::new(1, 1),
            opacity_1: 255,
            opacity_2: 255,
            drawing_in_progress: false,
            draw_receiver: None,
        }
    }
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            img_path_1: self.img_path_1.clone(),
            img_path_2: self.img_path_2.clone(),
            img_blur_1: self.img_blur_1,
            img_blur_2: self.img_blur_2,
            hue_rotation_1: self.hue_rotation_1,
            hue_rotation_2: self.hue_rotation_2,
            opacity_1: self.opacity_1,
            opacity_2: self.opacity_2,
            width: self.width,
            height: self.height,
            spacing: self.spacing,
            line_color: self.line_color,
            thickness: self.thickness,
            subdivisions: self.subdivisions,
            min_opacity: self.min_opacity,
            max_opacity: self.max_opacity,
            contamination: self.contamination,
            octaves: self.octaves,
            cutoff: self.cutoff,
            mode: self.mode,
            combine: self.combine,
            screen: self.screen,
            angle_scale: self.angle_scale,
            angle_factor: self.angle_factor,
            radius_scale: self.radius_scale,
            radius_factor: self.radius_factor,
            sort_key: self.sort_key,
            sort_by: self.sort_by,
            row_sort_order: self.row_sort_order,
            col_sort_order: self.col_sort_order,
            grain_scale: self.grain_scale,
            grain_factor: self.grain_factor,
            texture: None,
            img_1: self.img_1.clone(),
            img_2: self.img_2.clone(),
            img: self.img.clone(),
            drawing_in_progress: false,
            draw_receiver: None,
        }
    }
}
