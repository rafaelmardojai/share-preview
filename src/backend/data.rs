// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use url::Url;

use super::{Card, CardError, Image, Log, Social, scrape, Error};

#[derive(Debug, Default, Clone)]
pub struct Meta {
    pub name: Option<String>,
    pub property: Vec<String>,
    pub content: Option<String>,
    pub image: Option<Image>,
}

#[derive(Debug, Default, Clone)]
pub struct Data {
    pub url: String,
    pub title: Option<String>,
    pub favicon: Option<Image>,
    pub metadata: Vec<Meta>,
    pub body_images: Vec<Image>,
}

impl Data {
    pub async fn from_url(url: &Url) -> Result<Data, Error> {
        scrape(url).await
    }

    pub async fn get_card(&self, social: Social, logger: &impl Log) -> Result<Card, CardError> {
       Card::new(&self, social, logger).await
    }

    pub fn get_meta(&self, name: &str) -> Vec<&Meta> {
        let mut result = Vec::new();

        for meta in self.metadata.iter() {

            if meta.property.contains(&name.to_string()) {
                result.push(meta);
                continue;
            }

            if let Some(s) = &meta.name {
                if s == name {
                    result.push(meta);
                }
            }
        }

        result
    }
}
