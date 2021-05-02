// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env::temp_dir;
use std::error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use super::CLIENT;
use image;
use gtk::gdk_pixbuf;
use url::{Url, ParseError};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Image {
    pub url: String
}

impl Image {
    pub fn new(url: String) -> Image {
        Image {url}
    }
    
    pub fn normalize(&mut self, url: &Url) -> &mut Image {
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

    pub async fn fetch(&self, width: u32, height: u32) -> Result<gdk_pixbuf::Pixbuf, ImageError> {
        let mut resp = CLIENT.get(&self.url).await?;

        if resp.status().is_success() {
            let bytes = resp.body_bytes().await?;
            let format = image::guess_format(&bytes);

            let mut dir = temp_dir();
            let file_name = Uuid::new_v4().to_string();
            dir.push(file_name);

            match format {
                Ok(format) => {
                    let image = image::load_from_memory(&bytes).unwrap();
                    let thumbnail = image.resize_to_fill(width, height, image::imageops::FilterType::Triangle);
                    let dir = dir.to_str().unwrap();

                    match thumbnail.save_with_format(&dir, format) {
                        Ok(_) => {
                            match gdk_pixbuf::Pixbuf::from_file(&dir) {
                                Ok(pixbuf) => Ok(pixbuf),
                                Err(_) => Err(ImageError::PixbufError)
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
    PixbufError,
    Unexpected,
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ImageError::FetchError(ref e) => write!(f, "NetworkError:  {}", e),
            ImageError::InvalidFormat => write!(f, "InvalidFormat"),
            ImageError::PixbufError => write!(f, "PixbufError"),
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