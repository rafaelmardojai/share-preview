// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{RefCell, Cell},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Cursor,
};

use data_url::DataUrl;
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


#[derive(Debug, Clone)]
pub struct Image {
    pub base_url: Url,
    pub url: Url,
    pub was_relative: bool,
    pub bytes: RefCell<Option<Vec<u8>>>,
    pub format: Cell<Option<image::ImageFormat>>,
    pub width: Cell<Option<u32>>,
    pub height: Cell<Option<u32>>,
    pub size: Cell<Option<usize>>
}

impl Image {
    pub fn new(url: &String, base_url: &Url) -> Result<Image, ImageError> {

        // Check if url is valid and convert it to absolute if needed
        let (image_url, was_relative) = match Url::parse(&url) {
            // Return valid url
            Ok(url) => (url, false),
            // Convert to an absolute url if relative or fail the whole Image creation
            Err(e) => {
                match e {
                    ParseError::RelativeUrlWithoutBase => {
                        if let Ok(url) = base_url.join(&url) {
                            (url, true)
                        } else {
                            return Err(ImageError::UrlError(e));
                        }
                    },
                    _ => return Err(ImageError::UrlError(e))
                }
            }
        };


        Ok(
            Image {
                base_url: base_url.clone(),
                url: image_url,
                was_relative,
                bytes: RefCell::new(Option::default()),
                format: Cell::new(Option::default()),
                width: Cell::new(Option::default()),
                height: Cell::new(Option::default()),
                size: Cell::new(Option::default())
            }
        )
    }

    pub async fn fetch(&self) -> Result<Vec<u8>, ImageError> {
        //! Fetch image and get it's bytes
        //! After the first fetch the bytes, format and size are saved

        let saved_bytes = self.bytes.borrow().clone();

        match saved_bytes {
            // Return saved bytes without further fetching
            Some(bytes) => Ok(bytes),
            None => {
                // Fetch conditionally of the url scheme
                match self.url.scheme() {
                    "data" => {
                        let data = DataUrl::process(self.url.as_str())?;
                        let (body, _fragment) = data.decode_to_vec()?;

                        if let None = self.size.get() {
                            self.size.set(Some(body.len()));
                        }

                        self.bytes.replace(Some(body));
                        Ok(self.bytes.borrow().clone().unwrap())
                    },
                    _ => {
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

        // Calculate image dimensions if not available
        if let (None, None) = (self.width.get(), self.height.get()) {
            let (width, height) = async_std::task::spawn_blocking( move || -> Result<(u32, u32), ImageError> {
                let image = image::load_from_memory(&bytes)?;
                Ok((image.width(), image.height()))
            })
            .await?;

            self.width.set(Some(width));
            self.height.set(Some(height));
        }

        // Check if image meets the file size limits
        if let Some(size) = self.size.get() {
            if size > constraints.image_size {
                return Err(ImageError::TooHeavy{
                    actual: human_bytes(size as f64),
                    max: human_bytes(constraints.image_size as f64)
                });
            }
        }

        // Check if image meets the file format limitations
        if let Some(format) = self.format.get() {
            if !constraints.image_formats.contains(&format) {
                return Err(ImageError::Unsupported(gettext("Format is unsupported")));
            }
        }

        // Check if the image dimensions are allowed by some of the size kinds
        if let (Some(width), Some(height)) = (self.width.get(), self.height.get()) {

            let mut min_width: u32 = 0;
            let mut min_height: u32 = 0;

            // Loop image size kinds
            for kind in kinds.iter() {

                // Check if image match minimum dimensions for this kind
                let img_constraints = &social.image_size(kind);
                (min_width, min_height) = img_constraints.minimum;
                if width >= min_width && height >= min_height {
                    // Mastodon requieres the width of the image to be larger than its height for its extended preview
                    if let (Social::Mastodon, &SocialImageSizeKind::Large) = (social, kind) {
                        if height >= width {
                            continue;
                        }
                    }
                    // Kind matches! return it
                    return Ok(kind.to_owned());
                }
            }

            // No size kind matched, return too tiny error
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
    UrlError(ParseError),
    DataUrlError(data_url::DataUrlError),
    InvalidBase64(data_url::forgiving_base64::InvalidBase64),
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
            ImageError::UrlError(ref e) =>
                write!(f, "{}", gettext_f("Image Url Error: {info}", &[("info", &e.to_string())])),
            ImageError::DataUrlError(ref e) =>
                write!(f, "{}", gettext_f("Image Data Url Error: {info}", &[("info", &e.to_string())])),
            ImageError::InvalidBase64(ref e) =>
                write!(f, "{}", gettext_f("Image Data Invalid Base64: {info}", &[("info", &e.to_string())])),
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

impl From<data_url::DataUrlError> for ImageError {
    fn from(err: data_url::DataUrlError) -> ImageError {
        ImageError::DataUrlError(err)
    }
}

impl From<data_url::forgiving_base64::InvalidBase64> for ImageError {
    fn from(err: data_url::forgiving_base64::InvalidBase64) -> ImageError {
        ImageError::InvalidBase64(err)
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
