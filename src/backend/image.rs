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
    CardSize,
    Social,
    SocialImageSizeKind
};


#[derive(Debug, Default, Clone)]
pub struct Image {
    pub url: String,
    pub bytes: RefCell<Option<Vec<u8>>>,
    pub format: Cell<Option<image::ImageFormat>>,
    pub size: Cell<Option<usize>>
}

impl Image {
    pub fn new(url: String) -> Image {
        Image {
            url,
            bytes: RefCell::new(Option::default()),
            format: Cell::new(Option::default()),
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

    pub async fn thumbnail(
        &self,
        social: &Social,
        kinds: &Vec<SocialImageSizeKind>
    ) -> Result<(Vec<u8>, CardSize), ImageError> {

        let bytes = self.fetch().await?;
        let image = image::load_from_memory(&bytes)?;

        for kind in kinds.iter() {
            let (min_width, min_height) = social.image_size(kind);
            if image.width() >= min_width && image.height() >= min_height {
                let size = CardSize::from_social(kind);
                let (width, height) = size.image_size();
                let thumbnail = image.resize_to_fill(width, height, image::imageops::FilterType::Triangle);

                // Save to PNG so GTK can handle any format
                let mut thumbnail_bytes: Vec<u8> = Vec::new();
                thumbnail.write_to(&mut Cursor::new(&mut thumbnail_bytes), image::ImageFormat::Png)?;

                return Ok((thumbnail_bytes, size));
            }
        }

        Err(ImageError::TooTiny)
    }
}

#[derive(Debug)]
pub enum ImageError {
    FetchError(surf::Error),
    ImageError(image::error::ImageError),
    TooTiny,
    TooHeavy,
    Unsupported,
    Unexpected,
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ImageError::FetchError(ref e) => write!(f, "NetworkError: {}", e),
            ImageError::ImageError(ref e) => write!(f, "ImageError: {}", e),
            ImageError::TooTiny => write!(f, "TooTiny"),
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
