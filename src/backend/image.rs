// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use super::CLIENT;
use image;
use gtk::glib::Bytes;
use gtk::gdk::Texture;
use url::{Url, ParseError};

#[derive(Debug, Clone)]
pub struct Image {
    pub url: String
}

impl Image {
    pub fn new(url: String) -> Image {
        Image {url}
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

    pub async fn fetch(&self, width: u32, height: u32) -> Result<Texture, ImageError> {
        //! Fetch image and crop it to the given size
        //! return a Pixbuf to use in a GtkPicture

        let mut resp = CLIENT.get(&self.url).await?;

        if resp.status().is_success() {
            let bytes = resp.body_bytes().await?;
            let format = image::guess_format(&bytes)?;
            let image = image::load_from_memory_with_format(&bytes, format)?;
            let thumbnail = image.resize_to_fill(width, height, image::imageops::FilterType::Triangle);

            // Save to PNG so GTK can handle any format
            let mut thumbnail_bytes: Vec<u8> = Vec::new();
            thumbnail.write_to(&mut Cursor::new(&mut thumbnail_bytes), image::ImageFormat::Png)?;

            let gbytes = Bytes::from(&thumbnail_bytes);
            match Texture::from_bytes(&gbytes) {
                Ok(texture) => {
                    Ok(texture)
                },
                Err(_) => {
                    Err(ImageError::TextureError)
                }
            }
        } else {
            Err(ImageError::Unexpected)
        }
    }
}

#[derive(Debug)]
pub enum ImageError {
    FetchError(surf::Error),
    ImageError(image::error::ImageError),
    TextureError,
    Unexpected,
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ImageError::FetchError(ref e) => write!(f, "NetworkError: {}", e),
            ImageError::ImageError(ref e) => write!(f, "ImageError: {}", e),
            ImageError::TextureError => write!(f, "TextureError"),
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

impl error::Error for ImageError {
    fn description(&self) -> &str { "" }
}
