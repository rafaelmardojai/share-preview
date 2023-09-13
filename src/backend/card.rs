// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    error,
    fmt::{Display, Formatter, Result as FmtResult}
};

use gettextrs::gettext;

use crate::vec_of_strings;
use crate::i18n::gettext_f;
use super::{
    Data,
    Image,
    ImageError,
    Log,
    LogLevel,
    Social,
    social::{
        SocialMetaLookup,
        SocialConstraints,
        SocialImageSizeKind
    }
};

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
    pub favicon: Option<Vec<u8>>,
    pub description: Option<String>,
    pub image: Option<Vec<u8>>,
    pub size: CardSize,
    pub social: Social,
}

impl Card {
    pub async fn new(data: &Data, social: Social, logger: &impl Log) -> Result<Card, CardError> {
        //! Create a new Card from the found metadata based on the given Social

        let lookups: SocialMetaLookup = social.lookups();
        let constraints: SocialConstraints = social.constraints();
        let mut site = data.url.clone();
        let mut favicon: Option<Vec<u8>> = Option::None;
        let mut size = CardSize::default(); // Default card size
        let mut image: Option<Vec<u8>> = Option::None;
        let mut image_sizes: Vec<SocialImageSizeKind> = Vec::new();

        if let Some(fav) = &data.favicon {
            match fav.fetch().await {
                Ok(bytes) => {
                    favicon = Some(bytes);
                },
                Err(_) => {}
            };
        }

        // Prepare with already available data
        match social {
            Social::Facebook => {
                image_sizes.push(SocialImageSizeKind::Large);
                image_sizes.push(SocialImageSizeKind::Medium);
                site = site.to_uppercase();
            },
            Social::Discourse | Social::Mastodon => {
                image_sizes.push(SocialImageSizeKind::Small);
                // Mastodon uses og:site_name
                let look = vec_of_strings!["og:site_name"];
                if let Some(val) = Card::lookup_meta(&look, &data, None::<&dyn Log>) {
                    if !val.is_empty() {
                        site = val.to_string();
                        logger.log(LogLevel::Info, format!("{}: {}",
                            &social,
                            gettext_f("Found \"{name}\".", &[("name", "og:site_name")]))
                        );
                    } else {
                        logger.log(LogLevel::Warning, format!("{}: {}",
                            &social,
                            gettext_f("\"{name}\" is empty!", &[("name", "og:site_name")])
                        ));
                    }
                } else {
                    logger.log(LogLevel::Warning, format!("{}: {}",
                        &social,
                        gettext_f(
                            "Unable to find \"{name}\". Consider providing a \"{name}\" meta property.",
                            &[("name", "og:site_name")]
                        )
                    ));
                }
            },
            Social::Twitter => {}
        }

        // Get first available value from meta-tags to lookup
        let pre_title = Card::lookup_meta(&lookups.title, &data, Some(logger));
        let title = match &pre_title {
            Some(title) => title.to_string(),
            None => {
                logger.log(LogLevel::Warning, format!("{}: {}",
                    &social,
                    gettext("Unable to find a metadata for title!. Falling back to document title.")
                ));

                match &data.title {
                    Some(title) => {
                        title.to_string()
                    },
                    None => {
                        logger.log(LogLevel::Warning, format!("{}: {}",
                            &social,
                            gettext("Unable to find the document title!. Falling back to site url.")
                        ));

                        site.to_string()
                    },
                }
            }
        };

        // TODO: Get description from HTML for Facebook
        let description = Card::lookup_meta(&lookups.description, &data, Some(logger));

        let card_type = Card::lookup_meta(&lookups.kind, &data, Some(logger));
        if let Social::Twitter = &social {
            match card_type {
                Some(_) => {
                    // Check if it was a "twitter:card" and warn if not
                    // Change card size by the value of "twitter:card" meta-tag
                    let look = vec_of_strings!["twitter:card"];
                    if let Some(val) = Card::lookup_meta(&look, &data, None::<&dyn Log>) {
                        if val == "summary_large_image" {
                            image_sizes.push(SocialImageSizeKind::Large);
                        }

                        logger.log(LogLevel::Info, format!("{}: {}",
                            &social,
                            gettext_f("Found card of type \"{name}\".", &[("name", &val)])
                        ));
                    } else {
                        logger.log(LogLevel::Warning, format!("{}: {}",
                            &social,
                            gettext_f(
                                "Unable to find \"{name}\". Consider providing a \"{name}\" meta property.",
                                &[("name", "twitter:card")]
                            )
                        ));
                    }
                    image_sizes.push(SocialImageSizeKind::Medium);
                },
                None => {
                    // Return error if no card type is found for Twitter
                    logger.log(LogLevel::Error, format!("{}: {}",
                        &social,
                        gettext("Unable to find any valid card type.")
                    ));
                    return Err(CardError::TwitterNoCardFound);
                }
            }
        }

        // Return error if no basic data is found for Twitter
        if let (Social::Twitter, Option::None, Option::None) = (&social, &pre_title, &description) {
            logger.log(LogLevel::Error, format!("{}: {}",
                &social,
                gettext("Unable to find any valid title or description.")
            ));
            return Err(CardError::NotEnoughData);
        }

        match Card::lookup_meta_image(
            &social,
            &lookups.image,
            &data,
            &image_sizes,
            &constraints,
            logger
        ).await {
            Some((i, s)) => {
                image = Some(i);
                size = s;
            },
            None => {
                match &social {
                    Social::Discourse => {
                        logger.log(LogLevel::Warning, format!("{}: {}",
                            &social,
                            gettext("Unable to find a valid image in the metadata.")
                        ));
                    },
                    Social::Facebook => {
                        logger.log(LogLevel::Warning, format!("{}: {}",
                            &social,
                            gettext("Unable to find a valid image in the metadata, will look for images in the document body.")
                        ));
                        if data.body_images.len() > 0 {
                            match Card::lookup_fb_body_images(&social, &data.body_images, &constraints).await {
                                Some(i) => {
                                    image = Some(i);
                                    size = CardSize::Medium;
                                },
                                None => {
                                    logger.log(LogLevel::Warning, format!("{}: {}",
                                        &social,
                                        gettext("No valid images found in the document body.")
                                    ));
                                }
                            }
                        } else {
                            logger.log(LogLevel::Warning, format!("{}: {}",
                                &social,
                                gettext("No valid images found in the document body.")
                            ));
                        }
                    },
                    Social::Mastodon => {
                        logger.log(LogLevel::Warning, format!("{}: {}",
                            &social,
                            gettext("Unable to find a valid image in the metadata, will render an icon.")
                        ));
                    },
                    Social::Twitter => {
                        logger.log(LogLevel::Warning, format!("{}: {}",
                            &social,
                            gettext_f(
                                "Unable to find a valid image in the metadata, will render a \"{name}\" card with icon.",
                                &[("name", "summary")]
                            )
                        ));
                        size = CardSize::Medium;
                    }
                }
            }
        }

        Ok(Card {title, site, favicon, description, image, size, social})
    }

    pub fn lookup_meta(lookup: &Vec<String>, data: &Data, logger: Option<&(impl Log + ?Sized)>) -> Option<String> {
        for name in lookup.iter() {
            let occurrences = data.get_meta(name);

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

    pub async fn lookup_meta_image(
        social: &Social,
        lookup: &Vec<String>,
        data: &Data,
        kinds: &Vec<SocialImageSizeKind>,
        constraints: &SocialConstraints,
        logger: &impl Log
    ) -> Option<(Vec<u8>, CardSize)> {
        for name in lookup.iter() {
            let occurrences = data.get_meta(name);
            let mut valid: HashMap<SocialImageSizeKind, &Image> = HashMap::new();

            if occurrences.len() > 0 {
                logger.log(LogLevel::Debug, gettext_f(
                    "Looking for valid occurrences for \"{name}\"", &[("name", name)]
                ));
            }

            for meta in occurrences.iter() {
                if let Some(image) = &meta.image {
                    match image.check(social, kinds, constraints).await {
                        Ok(king) => {
                            if !valid.contains_key(&king) {
                                logger.log(LogLevel::Debug, gettext_f(
                                    "Image \"{url}\" met the requirements.", &[("url", &image.url)]
                                ));

                                let first_kind: bool = match kinds.first() {
                                    Some(k) => k == &king,
                                    None => false
                                };

                                valid.insert(king, image);

                                if first_kind {
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            match err {
                                ImageError::FetchError(_) => {
                                    logger.log(LogLevel::Debug, format!(
                                        "{}: \"{}\".", err, image.url
                                    ));
                                },
                                ImageError::RequestError(_) => {
                                    logger.log(LogLevel::Error, format!(
                                        "{}: \"{}\".", err, image.url
                                    ));
                                },
                                ImageError::TooHeavy{..} | ImageError::Unsupported(_) => {
                                    logger.log(LogLevel::Warning, format!("{}: {}",
                                        social,
                                        gettext_f(
                                            "Image \"{url}\" did not meet the requirements: {info}.",
                                            &[("url", &image.url), ("info", &err.to_string())]
                                        )
                                    ));
                                },
                                _ => {
                                    logger.log(LogLevel::Debug, format!("{}: {}",
                                        social,
                                        gettext_f(
                                            "Image \"{url}\" did not meet the requirements: {info}.",
                                            &[("url", &image.url), ("info", &err.to_string())]
                                        )
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            for kind in kinds {
                match valid.get(kind) {
                    Some(image) => {
                        let size = CardSize::from_social(kind);
                        let (width, height) = size.image_size();
                        match image.thumbnail(width, height).await {
                            Ok(bytes) => {
                                logger.log(LogLevel::Debug, gettext_f(
                                    "Image \"{url}\" processed successfully.", &[("url", &image.url)]
                                ));
                                return Some((bytes, size));
                            },
                            Err(err) => {
                                logger.log(LogLevel::Debug, gettext_f(
                                    "Failed to thumbnail \"{url}\": {info}.",
                                    &[("url", &image.url), ("info", &err.to_string())]
                                ));
                                continue;
                            }
                        };
                    }
                    None => ()
                }
            }

            logger.log(LogLevel::Debug, gettext_f(
                "No valid occurrences found for \"{name}\"!", &[("name", name)]
            ));
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
            match image.check(social, &kinds, constraints).await {
                Ok(kind) => {
                    let size = CardSize::from_social(&kind);
                    let (width, height) = size.image_size();
                    match image.thumbnail(width, height).await {
                        Ok(bytes) => Some(bytes),
                        Err(_) => continue
                    };
                }
                Err(_) => continue
            }
        }
        None
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
