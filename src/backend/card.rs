// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use super::{Data, Image};

macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

#[derive(Debug, Clone)]
pub enum Social {
    Facebook,
    Mastodon,
    Twitter,
}

#[derive(Debug, Clone)]
pub enum CardSize {
    Small, // Mastodon
    Medium, // Twitter sumary
    Large, // Twitter summary with large image || Facebook
}

impl CardSize {
    pub fn image_size(&self) -> (u32, u32) {
        match self {
            Self::Small => {
                (64, 64)
            },
            Self::Medium => {
                (125, 125)
            },
            Self::Large => {
                (500, 250)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub title: String,
    pub site: String,
    pub description: Option<String>,
    pub image: Option<Image>,
    pub size: CardSize,
    pub social: Social,
}

impl Card {
    pub fn new(data: &Data, social: Social) -> Card {
        //! Create a new Card from the found metadata based on the given Social
        
        let metadata = data.metadata.clone();
        let mut site = data.url.clone();
        let mut size = CardSize::Large; // Dafault card size

        // Default meta-tags to lookup the needed values
        let mut title_find = vec_of_strings!["og:title", "twitter:title", "title"];
        let mut description_find = vec_of_strings!["og:description", "twitter:description", "description"];
        let mut image_find = vec_of_strings!["og:image", "twitter:image", "twitter:image:src"];

        // Change meta-tags to lookup and default values by the given Social:
        match social {
            Social::Facebook => {
                site = site.to_uppercase();
            },
            Social::Mastodon => {
                image_find = vec_of_strings!["og:image"];
                size = CardSize::Small; // Mastodon always use a small card size

                if metadata.contains_key("og:site_name") {
                    site = metadata.get("og:site_name").unwrap().to_string();
                }                
            },
            Social::Twitter => {
                title_find = vec_of_strings!["twitter:title", "og:title", "title"];
                description_find = vec_of_strings!["twitter:description", "og:description", "description"];
                image_find = vec_of_strings!["twitter:image", "twitter:image:src", "og:image"];

                // Change card size by the value of "twitter:card" meta-tag
                if metadata.contains_key("twitter:card") {
                    match metadata.get("twitter:card").unwrap().as_str() {
                        "summary_large_image" => (), // Do nothing
                        "summary" => size = CardSize::Medium,
                        _ => ()
                    }
                } else {
                    size = CardSize::Medium;
                }
            }
        }

        // Get first available value from meta-tags to lookup
        let pre_title = Card::get_correct_tag(&title_find, &metadata);
        let title = match pre_title { // Convert image String to a Image struct:
            Some(title) => title,
            None => site.to_string()
        };
        let description = Card::get_correct_tag(&description_find, &metadata);
        let pre_image = Card::get_correct_tag(&image_find, &metadata);        
        let image = match pre_image { // Convert image String to a Image struct:
            Some(url) => Some(Image::new(url)),
            None => (None)
        };

        Card {title, site, description, image, size, social}
    }

    pub fn get_correct_tag(
            list: &Vec<String>,
            metadata: &HashMap<String, String>) -> Option<String> {
        //! Get first available value from meta-tags to lookup

        for term in list.iter() {
            match metadata.get(term) {
                Some(content) => {
                    let content = content.clone();
                    return Some(content);
                },
                None => ()
            }
        }

        None
    }
}