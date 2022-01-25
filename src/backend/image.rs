// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

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
            let format = image::guess_format(&bytes);

            match format {
                Ok(format) => {
                    let image = image::load_from_memory_with_format(&bytes, format).unwrap();
                    // Resize and crop image to the give size:
                    let thumbnail = image.resize_to_fill(width, height, image::imageops::FilterType::Triangle);
                    let mut thumbnail_bytes = Vec::new();

                    match thumbnail.write_to(&mut thumbnail_bytes, format) {
                        Ok(_) => {
                            let bytes = Bytes::from(&thumbnail_bytes);
                            match Texture::from_bytes(&bytes) {
                                Ok(texture) => {
                                    Ok(texture)
                                },
                                Err(_) => {
                                    Err(ImageError::TextureError)
                                }
                            }
                        },
                        Err(_) => Err(ImageError::Unexpected)
                    }
                },
                Err(_) => Err(ImageError::InvalidFormat)
            }
        } else {
            Err(ImageError::Unexpected)
        }
    }
}

#[derive(Debug)]
pub enum ImageError {
    FetchError(surf::Error),
    InvalidFormat,
    TextureError,
    Unexpected,
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ImageError::FetchError(ref e) => write!(f, "NetworkError: {}", e),
            ImageError::InvalidFormat => write!(f, "InvalidFormat"),
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

impl error::Error for ImageError {
    fn description(&self) -> &str { "" }
}
