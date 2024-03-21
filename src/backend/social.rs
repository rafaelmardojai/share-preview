// Copyright 2023 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr
};

use image::ImageFormat;

use crate::vec_of_strings;

const NAMES: [&str; 2] =  [
    "og:title", "title"
];
const DESCRIPTIONS: [&str; 2] = [
    "og:description", "description"
];
const IMAGES: [&str; 1] = [
    "og:image"
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
#[derive(Debug, Clone, PartialEq)]
pub enum Social {
    Discourse,
    Facebook,
    LinkedIn,
    Mastodon,
    Twitter,
}

impl Display for Social {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Social::Discourse => write!(f, "Discourse"),
            Social::Facebook => write!(f, "Facebook"),
            Social::LinkedIn => write!(f, "LinkedIn"),
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
            "Discourse" => Ok(Self::Discourse),
            "Facebook" => Ok(Self::Facebook),
            "LinkedIn" => Ok(Self::LinkedIn),
            "Mastodon" => Ok(Self::Mastodon),
            "Twitter" => Ok(Self::Twitter),
            _ => Ok(Self::Discourse)
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
                Self::LinkedIn => vec_of_strings!["og:description"],
                Self::Twitter => vec_of_strings!["twitter:description", "og:description"],
                _ => DESCRIPTIONS.iter().map(|s| s.to_string()).collect::<Vec<String>>()
            },
            image: match self {
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

    pub fn image_size(&self, kind: &SocialImageSizeKind) -> SocialImageConstraints {
        SocialImageConstraints {
            minimum: match self {
                Self::Discourse => (50, 50),
                Self::Facebook => {
                    match kind {
                        SocialImageSizeKind::Large => (600, 315),
                        _ => (200, 200)
                    }
                },
                Self::LinkedIn => (20, 20),
                Self::Mastodon => (50, 50),
                Self::Twitter => {
                    match kind {
                        SocialImageSizeKind::Large => (300, 157),
                        _ => (144, 144)
                    }
                }
            },
            recommended: match self {
                Self::Discourse => (50, 50),
                Self::Facebook => {
                    match kind {
                        SocialImageSizeKind::Large => (600, 315),
                        _ => (200, 200)
                    }
                },
                Self::LinkedIn => (600, 315),
                Self::Mastodon => (100, 100),
                Self::Twitter => {
                    match kind {
                        SocialImageSizeKind::Large => (300, 157),
                        _ => (144, 144)
                    }
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
    /// Image maximum size
    pub image_size: usize,
    /// Image allowed formats
    pub image_formats: Vec<ImageFormat>,
}

#[derive(Debug, Clone)]
pub struct SocialImageConstraints {
    /// Image minimum dimensions
    pub minimum: (u32, u32),
    /// Image recommended dimensions
    pub recommended: (u32, u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SocialImageSizeKind {
    Small,
    Medium,
    Large,
}
