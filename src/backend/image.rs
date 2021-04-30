// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Error, CLIENT};
use gtk::{gio, glib, gdk_pixbuf};
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

    pub async fn fetch(&self) -> Result<gdk_pixbuf::Pixbuf, Error> {
        let mut resp = CLIENT.get(&self.url).await?;
        let bytes = resp.body_bytes().await?;
        let bytes = glib::Bytes::from(&bytes);
        let stream = gio::MemoryInputStream::from_bytes(&bytes);

        Ok(gdk_pixbuf::Pixbuf::from_stream(&stream, gio::NONE_CANCELLABLE).unwrap())
    }
}
