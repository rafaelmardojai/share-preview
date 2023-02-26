// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::vec_of_strings;
use super::{
    Data,
    Image,
    ImageError,
    Social,
    social::{
        SocialMetaLookup,
        SocialConstraints,
        SocialImageSizeKind
    }
};

use std::error;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Default, Clone)]
pub enum CardSize {
    #[default]
    Small,
    Medium,
    Large,
}

impl CardSize {
    pub fn from_social(kind: &SocialImageSizeKind) -> CardSize {
        match kind {
            SocialImageSizeKind::Small => CardSize::Small,
            SocialImageSizeKind::Medium => CardSize::Medium,
            SocialImageSizeKind::Large => CardSize::Large
        }
    }

    pub fn image_size(&self) -> (u32, u32) {
        match self {
            Self::Small => (64, 64),
            Self::Medium => (125, 125),
            Self::Large => (500, 250)
        }
    }

    pub fn icon_size(&self) -> i32 {
        match self {
            Self::Small => 32,
            Self::Medium => 48,
            Self::Large => 64
        }
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub title: String,
    pub site: String,
    pub description: Option<String>,
    pub image: Option<Vec<u8>>,
    pub size: CardSize,
    pub social: Social,
}

impl Card {
    pub async fn new(data: &Data, social: Social) -> Result<Card, CardError> {
        //! Create a new Card from the found metadata based on the given Social

        let lookups: SocialMetaLookup = social.lookups();
        let constraints: SocialConstraints = social.constraints();
        let mut site = data.url.clone();
        let mut size = CardSize::default(); // Default card size
        let mut image: Option<Vec<u8>> = Option::None;
        let mut image_sizes: Vec<SocialImageSizeKind> = Vec::new();

        // Prepare with already available data
        match social {
            Social::Facebook => {
                image_sizes.push(SocialImageSizeKind::Large);
                image_sizes.push(SocialImageSizeKind::Medium);
                site = site.to_uppercase();
            },
            Social::Mastodon => {
                image_sizes.push(SocialImageSizeKind::Small);
                // Mastodon uses og:site_name
                let look = vec_of_strings!["og:site_name"];
                if let Some(val) = Card::lookup_meta(&look, &data) {
                    if !val.is_empty() {
                        site = val.to_string();
                    }
                }
            },
            Social::Twitter => {
                // Change card size by the value of "twitter:card" meta-tag
                let look = vec_of_strings!["twitter:card"];
                if let Some(val) = Card::lookup_meta(&look, &data) {
                    if val == "summary_large_image" {
                        image_sizes.push(SocialImageSizeKind::Large);
                    }
                }
                image_sizes.push(SocialImageSizeKind::Medium);
            }
        }

        // Get first available value from meta-tags to lookup
        let pre_title = Card::lookup_meta(&lookups.title, &data);
        let title = match &pre_title {
            Some(title) => title.to_string(),
            None => {
                match &data.title {
                    Some(title) => title.to_string(),
                    None => site.to_string(),
                }
            }
        };

        // TODO: Get description from HTML for Facebook
        let description = Card::lookup_meta(&lookups.description, &data);

        let card_type = Card::lookup_meta(&lookups.kind, &data);

        // Return error if no basic data is found for Twitter
        if let (Social::Twitter, Option::None, Option::None) = (&social, &pre_title, &description) {
            return Err(CardError::NotEnoughData);
        }
        // Return error if no card type is found for Twitter
        if let (Social::Twitter, Option::None) = (&social, &card_type) {
            return Err(CardError::TwitterNoCardFound);
        }

        match Card::lookup_meta_image(
            &social,
            &lookups.image,
            &data,
            &image_sizes,
            &constraints
        ).await {
            Some((i, s)) => {
                image = Some(i);
                size = s;
            },
            None => {
                if let Social::Twitter = &social {
                    size = CardSize::Medium;
                }

                if let Social::Facebook = &social {
                    if data.body_images.len() > 0 {
                        match Card::lookup_fb_body_images(&social, &data.body_images, &constraints).await {
                            Some(i) => {
                                image = Some(i);
                                size = CardSize::Medium;
                            },
                            _ => ()
                        }
                    }
                }
            }
        }

        Ok(Card {title, site, description, image, size, social})
    }

    pub fn lookup_meta(lookup: &Vec<String>, data: &Data) -> Option<String> {
        for name in lookup.iter() {
            let occurrences = data.get_meta(name);

            if let Some(meta) = occurrences.first() {
                if let Some(val) = &meta.content {
                    if !val.is_empty() {
                        return Some(val.to_string());
                    } else {
                        continue
                    }
                }
            };
        }
        None
    }

    pub async fn lookup_meta_image(
        social: &Social,
        lookup: &Vec<String>,
        data: &Data,
        kinds: &Vec<SocialImageSizeKind>,
        constraints: &SocialConstraints
    ) -> Option<(Vec<u8>, CardSize)> {
        for name in lookup.iter() {
            let occurrences = data.get_meta(name);

            for meta in occurrences.iter() {
                if let Some(image) = &meta.image {
                    match Card::try_get_image(social, image, kinds, constraints).await {
                        Ok((bytes, size)) => {
                            return Some((bytes, size));
                        }
                        Err(_) => continue
                    }
                }
            }
        }
        None
    }

    pub async fn lookup_fb_body_images(
        social: &Social,
        images: &Vec<Image>,
        constraints: &SocialConstraints
    ) -> Option<Vec<u8>> {
        let kinds = [SocialImageSizeKind::Medium].to_vec();

        for image in images.iter() {
            match Card::try_get_image(social, &image, &kinds, constraints).await {
                Ok((bytes, _)) => {
                    return Some(bytes)
                }
                Err(_) => continue
            }
        }
        None
    }

    pub async fn try_get_image(
        social: &Social,
        image: &Image,
        kinds: &Vec<SocialImageSizeKind>,
        _constraints: &SocialConstraints
    ) -> Result<(Vec<u8>, CardSize), ImageError> {
        match image.thumbnail(social, kinds).await {
            Ok((bytes, size)) => {
                Ok((bytes, size))
            },
            Err(e) => Err(e)
        }
    }
}

#[derive(Debug)]
pub enum CardError {
    NotEnoughData,
    TwitterNoCardFound
}

impl Display for CardError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            CardError::NotEnoughData => write!(f, "NotEnoughData"),
            CardError::TwitterNoCardFound => write!(f, "TwitterNoCardFound"),
        }
    }
}

impl error::Error for CardError {}
