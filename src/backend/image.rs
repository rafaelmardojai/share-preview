// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    cell::{RefCell, Cell},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Cursor,
};

use image;
use url::{Url, ParseError};

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
        //! Fetch image ans get it's bytes
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
                        self.size.set(resp.len());
                    }

                    self.bytes.replace(Some(bytes));
                    Ok(self.bytes.borrow().clone().unwrap())
                } else {
                    Err(ImageError::Unexpected)
                }
            }
        }
    }

    pub async fn check(
        &self,
        social: &Social,
        kinds: &Vec<SocialImageSizeKind>,
        _constraints: &SocialConstraints
    ) -> Result<SocialImageSizeKind, ImageError> {
        if let (None, None) = (self.width.get(), self.height.get()) {

            let bytes = self.fetch().await?;

            let (width, height) = async_std::task::spawn_blocking( move || -> Result<(u32, u32), ImageError> {
                let image = image::load_from_memory(&bytes)?;
                Ok((image.width(), image.height()))
            })
            .await?;

            self.width.set(Some(width));
            self.height.set(Some(height));
        }

        if let (Some(width), Some(height)) = (self.width.get(), self.height.get()) {
            for kind in kinds.iter() {
                let (min_width, min_height) = social.image_size(kind);
                if width >= min_width && height >= min_height {
                    return Ok(kind.to_owned());
                }
            }

            let sizes: (u32, u32) = match kinds.last() {
                Some(kind) => social.image_size(kind),
                None => (0, 0)
            };

            Err(ImageError::TooTiny(format!("{}x{}", sizes.0, sizes.1)))
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
}

#[derive(Debug)]
pub enum ImageError {
    FetchError(surf::Error),
    ImageError(image::error::ImageError),
    TooTiny(String),
    TooHeavy,
    Unsupported,
    Unexpected,
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ImageError::FetchError(ref e) => write!(f, "NetworkError: {}", e),
            ImageError::ImageError(ref e) => write!(f, "ImageError: {}", e),
            ImageError::TooTiny(ref s) => write!(f, "Image is too tiny, minimum size is {}", s),
            ImageError::TooHeavy => write!(f, "TooHeavy"),
            ImageError::Unsupported => write!(f, "Unsupported"),
            ImageError::Unexpected => write!(f, "UnexpectedError"),
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
