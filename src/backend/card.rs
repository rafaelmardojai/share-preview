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
            Social::LinkedIn => {
                image_sizes.push(SocialImageSizeKind::Large);
            },
            Social::Discourse | Social::Mastodon => {
                image_sizes.push(SocialImageSizeKind::Small);
                // Mastodon uses og:site_name
                let look = vec_of_strings!["og:site_name"];
                if let Some(val) = data.lookup_meta(&look, None::<&dyn Log>) {
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
        let pre_title = data.lookup_meta(&lookups.title, Some(logger));
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
        let description = data.lookup_meta(&lookups.description, Some(logger));

        if let Some(text) = &description {
            if let Social::LinkedIn = &social {
                if &text.chars().count() < &100 {
                    logger.log(LogLevel::Warning, format!("{}: {}",
                        &social,
                        gettext_f("The description should be at least \"{count}\" characters long.", &[("count", "100")])
                    ));
                }
            }
        }

        let card_type = data.lookup_meta(&lookups.kind, Some(logger));
        if let Social::Twitter = &social {
            match card_type {
                Some(_) => {
                    // Check if it was a "twitter:card" and warn if not
                    // Change card size by the value of "twitter:card" meta-tag
                    let look = vec_of_strings!["twitter:card"];
                    if let Some(val) = data.lookup_meta(&look, None::<&dyn Log>) {
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

        // Get possible images
        let include_body_imgs = match &social {
            Social::LinkedIn | Social::Facebook => true,
            _ => false
        };
        let images = data.lookup_meta_images(&lookups.image, include_body_imgs);


        match Card::lookup_image(
            &social,
            images,
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
                    Social::LinkedIn | Social::Facebook => {
                        logger.log(LogLevel::Warning, format!("{}: {}",
                            &social,
                            gettext("Unable to find a valid image in the metadata or document body.")
                        ));
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

    pub async fn lookup_image(
        social: &Social,
        images: Vec<&Image>,
        kinds: &Vec<SocialImageSizeKind>,
        constraints: &SocialConstraints,
        logger: &impl Log
    ) -> Option<(Vec<u8>, CardSize)> {

        // Check what images are minimally viable for the given kinds
        let mut valid: HashMap<SocialImageSizeKind, Vec<&Image>> = HashMap::new();
        for image in images.iter() {
            match image.check(&social, kinds, constraints).await {
                Ok(kind) => {
                    logger.log(LogLevel::Debug, gettext_f(
                        "Image \"{url}\" met the requirements.", &[("url", &image.url)]
                    ));

                    if !valid.contains_key(&kind) {
                        valid.insert(kind.clone(), Vec::default());
                    }

                    let img_vec = valid.get_mut(&kind).unwrap();
                    img_vec.push(image);
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

        for kind in kinds {
            if valid.contains_key(&kind) {
                let (_, img_vec) = valid.get_key_value(&kind).unwrap();

                let (width, height) = social.image_size(kind).recommended;
                let mut prev_higher_size: (u32, u32) = (0, 0);
                let mut prev_higher: Option<&&Image> = None;

                for image in img_vec {
                    let (image_width, image_height) = image.size();
                    if image_width >= width && image_height >= height {

                        if prev_higher_size == (0, 0) || image_width >= prev_higher_size.0 && image_height >= prev_higher_size.1 {
                            prev_higher_size = (image_width, image_height);
                            prev_higher = Some(image);
                        }
                    }
                }

                if let Some(image) = prev_higher {
                    let size = CardSize::from_social(kind);
                    let (cut_width, cut_height) = size.image_size();
                    match image.thumbnail(cut_width, cut_height).await {
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
