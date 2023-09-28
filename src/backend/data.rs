// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use url::Url;

use crate::i18n::gettext_f;
use super::{Card, CardError, Image, Log, LogLevel, Social, scrape, Error};

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

    /// Get a Metas matching a name or property
    ///
    /// * `name` - The name or a property of the meta to get
    ///
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

    /// Similar to get_meta() but directly looks for images
    ///
    /// * `name` - The name or a property of the meta to get
    ///
    pub fn get_meta_image(&self, name: &str) -> Vec<&Image> {
        let mut result = Vec::new();

        for meta in self.metadata.iter() {

            if meta.property.contains(&name.to_string()) {
                if let Some(image) = &meta.image {
                    result.push(image);
                }
                continue;
            }

            if let Some(s) = &meta.name {
                if s == name {
                    if let Some(image) = &meta.image {
                        result.push(image);
                    }
                }
            }
        }

        result
    }

    /// Look for the first matching meta from a strings vector
    ///
    /// This method logs the lookup process if logger is provider
    ///
    /// * `lookup` - Meta names and properties to lookup
    /// * `logger` - Log object to log the lookup process
    ///
    pub fn lookup_meta(&self, lookup: &Vec<String>, logger: Option<&(impl Log + ?Sized)>) -> Option<String> {
        for name in lookup.iter() {
            let occurrences = self.get_meta(name);

            if let Some(meta) = occurrences.first() {
                if let Some(val) = &meta.content {
                    if !val.is_empty() {
                        if let Some(log) = logger {
                            log.log(LogLevel::Debug, gettext_f(
                                "Found a valid occurrence for \"{name}\" with value \"{value}\".",
                                &[("name", name), ("value", val)]
                            ));
                        }
                        return Some(val.to_string());
                    } {
                        if let Some(log) = logger {
                            log.log(LogLevel::Warning, gettext_f(
                                "\"{name}\" is empty!", &[("name", name)]
                            ));
                        }
                        continue;
                    }
                }
            };

            if let Some(log) = logger {
                log.log(LogLevel::Debug, gettext_f(
                    "No occurrences found for \"{name}\"!", &[("name", name)]
                ));
            }
        }
        None
    }

    /// Similar to lookup_meta but only looks for images
    ///
    /// This method unlike lookup_meta does not log the process
    ///
    /// * `lookup` - Meta names and properties to lookup
    ///
    pub fn lookup_meta_images(&self, lookup: &Vec<String>) -> Vec<&Image> {

        let mut images: Vec<&Image> = Vec::new();
        for name in lookup.iter() {
            let occurrences = self.get_meta_image(name);
            images.extend(occurrences);
        }

        images
    }


    /// Gets Data's body_images with a return type matching lookup_meta_images
    ///
    /// It also truncates the vector to a max to avoid long loads
    ///
    pub fn get_body_images(&self, len: usize) -> Vec<&Image> {
        let mut new = self.body_images.iter().map(|img| {
            img
        }).collect::<Vec<&Image>>();

        new.truncate(len);

        return new;
    }
}
