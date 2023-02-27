// Copyright 2023 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr
};

use image::ImageFormat;

use crate::vec_of_strings;

const NAMES: [&str; 3] =  [
    "og:title", "twitter:title", "title"
];
const DESCRIPTIONS: [&str; 3] = [
    "og:description", "twitter:description", "description"
];
const IMAGES: [&str; 3] = [
    "og:image", "twitter:image", "twitter:image:src"
];
const KINDS: [&str; 1] = ["og:type"];
const IMAGE_FORMATS: [ImageFormat; 4] = [
    ImageFormat::Png,
    ImageFormat::Jpeg,
    ImageFormat::Gif,
    ImageFormat::WebP
];
const MAX_SIZE: usize = 5e+6 as usize;

/// Enumerates supported platforms
#[derive(Debug, Clone)]
pub enum Social {
    Facebook,
    Mastodon,
    Twitter,
}

impl Display for Social {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Social::Facebook => write!(f, "Facebook"),
            Social::Mastodon => write!(f, "Mastodon"),
            Social::Twitter => write!(f, "Twitter"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SocialParseError;

impl FromStr for Social {
    type Err = SocialParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Facebook" => Ok(Self::Facebook),
            "Mastodon" => Ok(Self::Mastodon),
            "Twitter" => Ok(Self::Twitter),
            _ => Ok(Self::Facebook)
        }
    }
}

impl Social {
    pub fn lookups(&self) -> SocialMetaLookup {
        SocialMetaLookup {
            title: match self {
                Self::Twitter => vec_of_strings!["twitter:title", "og:title", "title"],
                _ => NAMES.iter().map(|s| s.to_string()).collect::<Vec<String>>()
            },
            description: match self {
                Self::Twitter => vec_of_strings!["twitter:description", "og:description"],
                _ => DESCRIPTIONS.iter().map(|s| s.to_string()).collect::<Vec<String>>()
            },
            image: match self {
                Self::Mastodon => vec_of_strings!["og:image"],
                Self::Twitter => vec_of_strings!["twitter:image", "twitter:image:src", "og:image"],
                _ => IMAGES.iter().map(|s| s.to_string()).collect::<Vec<String>>()
            },
            kind: match self {
                Self::Twitter => vec_of_strings!["twitter:card", "og:type"],
                _ => KINDS.iter().map(|s| s.to_string()).collect::<Vec<String>>()
            }
        }
    }

    pub fn constraints(&self) -> SocialConstraints {
        SocialConstraints {
            image_size: match self {
                Self::Facebook => 8e+6 as usize,
                _ => MAX_SIZE
            },
            image_formats: match self {
                _ => IMAGE_FORMATS.to_vec()
            }
        }
    }

    pub fn image_size(&self, kind: &SocialImageSizeKind) -> (u32, u32) {
        match self {
            Self::Facebook => {
                match kind {
                    SocialImageSizeKind::Large => { (600, 315) },
                    _ => { (200, 200) }
                }
            },
            Self::Mastodon => {
                (100, 100)
            },
            Self::Twitter => {
                match kind {
                    SocialImageSizeKind::Large => { (300, 300) },
                    _ => { (144, 144) }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SocialMetaLookup {
    pub title: Vec<String>,
    pub description: Vec<String>,
    pub image: Vec<String>,
    pub kind: Vec<String>
}

#[derive(Debug, Clone)]
pub struct SocialConstraints {
    pub image_size: usize,
    pub image_formats: Vec<ImageFormat>,
}

#[derive(Debug, Clone)]
pub enum SocialImageSizeKind {
    Small,
    Medium,
    Large,
}
