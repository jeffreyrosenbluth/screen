use egui::{ColorImage, TextureHandle};
use image::{
    imageops::{self, FilterType},
    RgbaImage,
};

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
pub enum MapColor {
    Lightness,
    RedGreen,
    YellowBlue,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum BlendMode {
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
    pub color_map: MapColor,
    pub sort_key: SortKey,
    pub sort_by: SortBy,
    pub row_sort_order: SortOrder,
    pub col_sort_order: SortOrder,

    #[serde(skip)]
    pub texture: Option<TextureHandle>,

    #[serde(skip)]
    pub img_1: RgbaImage,

    #[serde(skip)]
    pub img_2: RgbaImage,
}

impl Default for App {
    fn default() -> Self {
        Self {
            img_path_1: None,
            img_path_2: None,
            img_blur_1: 100.0,
            img_blur_2: 75.0,
            hue_rotation_1: 0,
            hue_rotation_2: 0,
            width: 4032,
            height: 3024,
            spacing: 25.0,
            line_color: LineColor::Black,
            thickness: 1.0,
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
            color_map: MapColor::Lightness,
            sort_key: SortKey::Lightness,
            sort_by: SortBy::Row,
            row_sort_order: SortOrder::Ascending,
            col_sort_order: SortOrder::Ascending,
            texture: None,
            img_1: RgbaImage::new(1, 1),
            img_2: RgbaImage::new(1, 1),
        }
    }
}
