// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{RefCell, Cell},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Cursor,
};

use gettextrs::gettext;
use human_bytes::human_bytes;
use image;
use url::{Url, ParseError};

use crate::i18n::gettext_f;
use super::{
    CLIENT,
    Social,
    SocialImageSizeKind,
    SocialConstraints
};


#[derive(Debug, Default, Clone)]
pub struct Image {
    pub url: String,
    pub bytes: RefCell<Option<Vec<u8>>>,
    pub format: Cell<Option<image::ImageFormat>>,
    pub width: Cell<Option<u32>>,
    pub height: Cell<Option<u32>>,
    pub size: Cell<Option<usize>>
}

impl Image {
    pub fn new(url: String) -> Image {
        Image {
            url,
            bytes: RefCell::new(Option::default()),
            format: Cell::new(Option::default()),
            width: Cell::new(Option::default()),
            height: Cell::new(Option::default()),
            size: Cell::new(Option::default())
        }
    }

    pub fn normalize(&mut self, url: &Url) -> &mut Image {
        //! Normalize image URL if is relative

        if let Err(e) = Url::parse(&self.url) {
            match e {
                ParseError::RelativeUrlWithoutBase => {
                    if let Ok(url) = url.join(&self.url) {
                        self.url = url.to_string();
                    }
                },
                _ => (),
            }
        }
        self
    }

    pub async fn fetch(&self) -> Result<Vec<u8>, ImageError> {
        //! Fetch image and get it's bytes
        //! After the first fetch the bytes, format and size are saved

        let saved_bytes = self.bytes.borrow().clone();

        match saved_bytes {
            Some(bytes) => Ok(bytes),
            None => {
                let mut resp = CLIENT.get(&self.url).await?;

                if resp.status().is_success() {
                    let bytes = resp.body_bytes().await?;
                    let format = image::guess_format(&bytes)?;

                    if let None = self.format.get() {
                        self.format.set(Some(format));
                    }
                    if let None = self.size.get() {
                        self.size.set(Some(bytes.len()));
                    }

                    self.bytes.replace(Some(bytes));
                    Ok(self.bytes.borrow().clone().unwrap())
                } else {
                    Err(ImageError::RequestError(resp.status().canonical_reason()))
                }
            }
        }
    }

    /// Checks if the images meet the constrains of any of the given kinds.
    pub async fn check(
        &self,
        social: &Social,
        kinds: &Vec<SocialImageSizeKind>,
        constraints: &SocialConstraints
    ) -> Result<SocialImageSizeKind, ImageError> {
        let bytes = self.fetch().await?;

        if let (None, None) = (self.width.get(), self.height.get()) {
            let (width, height) = async_std::task::spawn_blocking( move || -> Result<(u32, u32), ImageError> {
                let image = image::load_from_memory(&bytes)?;
                Ok((image.width(), image.height()))
            })
            .await?;

            self.width.set(Some(width));
            self.height.set(Some(height));
        }

        if let Some(size) = self.size.get() {
            if size > constraints.image_size {
                return Err(ImageError::TooHeavy{
                    actual: human_bytes(size as f64),
                    max: human_bytes(constraints.image_size as f64)
                });
            }
        }

        if let Some(format) = self.format.get() {
            if !constraints.image_formats.contains(&format) {
                return Err(ImageError::Unsupported(gettext("Format is unsupported")));
            }
        }

        if let (Some(width), Some(height)) = (self.width.get(), self.height.get()) {

            let mut min_width: u32 = 0;
            let mut min_height: u32 = 0;

            for kind in kinds.iter() {
                let img_constraints = &social.image_size(kind);
                (min_width, min_height) = img_constraints.minimum;
                if width >= min_width && height >= min_height {
                    return Ok(kind.to_owned());
                }
            }

            Err(ImageError::TooTiny{
                actual: format!("{}×{}px", width, height),
                min: format!("{}×{}px", min_width, min_height)
            })
        } else {
            Err(ImageError::Unexpected)
        }
    }

    pub async fn thumbnail(
        &self,
        width: u32,
        height: u32
    ) -> Result<Vec<u8>, ImageError> {
        let bytes = self.fetch().await?;

        let thumbnail_bytes = async_std::task::spawn_blocking( move || -> Result<Vec<u8>, ImageError> {
            let mut thumbnail_bytes: Vec<u8> = Vec::new();
            let image = image::load_from_memory(&bytes)?;

            // Create thumbnail
            let thumbnail = image.resize_to_fill(
                width,
                height,
                image::imageops::FilterType::Triangle
            );

            // Save to PNG so GTK can handle any format
            thumbnail.write_to(&mut Cursor::new(&mut thumbnail_bytes), image::ImageFormat::Png)?;
            Ok(thumbnail_bytes)
        })
        .await?;

        Ok(thumbnail_bytes)
    }

    /// Return the image size as a tuple
    ///
    /// If check() was never called before, this will just return (0, 0)
    ///
    pub fn size(&self) -> (u32, u32) {
        match (self.width.get(), self.height.get()) {
            (Some(width), Some(height)) => (width, height),
            _ => (0, 0)
        }
    }
}

#[derive(Debug)]
pub enum ImageError {
    FetchError(surf::Error),
    RequestError(&'static str),
    ImageError(image::error::ImageError),
    TooTiny{
        actual: String,
        min: String
    },
    TooHeavy{
        actual: String,
        max: String
    },
    Unsupported(String),
    Unexpected,
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ImageError::FetchError(ref e) =>
                write!(f, "{}", gettext_f("Network Error: {info}", &[("info", &e.to_string())])),
            ImageError::RequestError(ref s) =>
                write!(f, "{}", gettext_f("Request Error: {info}", &[("info", s)])),
            ImageError::ImageError(ref e) =>
                write!(f, "{}", gettext_f("Image Error: {info}", &[("info", &e.to_string())])),
            ImageError::TooTiny{ref actual, ref min} =>
                write!(f, "{}", gettext_f(
                    "Image is too tiny ({actual}), minimum dimensions are {min}",
                    &[("actual", actual), ("min", min)]
                )),
            ImageError::TooHeavy{ref actual, ref max} =>
                write!(f, "{}", gettext_f(
                    "Image is too heavy ({actual}), max size is {max}",
                    &[("actual", actual), ("max", max)]
                )),
            ImageError::Unsupported(ref s) =>
                write!(f, "{}", gettext_f("Image is unsupported: {info}", &[("info", s)])),
            ImageError::Unexpected =>
                write!(f, "{}", gettext("Unexpected Error")),
        }
    }
}

impl From<surf::Error> for ImageError {
    fn from(err: surf::Error) -> ImageError {
        ImageError::FetchError(err)
    }
}

impl From<image::error::ImageError> for ImageError {
    fn from(err: image::error::ImageError) -> ImageError {
        ImageError::ImageError(err)
    }
}

impl Error for ImageError {
    fn description(&self) -> &str { "" }
}
