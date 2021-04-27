use std::fmt::{Display, Formatter, Result as FmtResult};
use std::collections::HashMap;
use std::error;

use surf;
use url::{Url, ParseError};

macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

#[derive(Debug)]
pub enum Error {
    NetworkError(surf::Error),
    Unexpected,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Error::NetworkError(ref e) => write!(f, "NetworkError:  {}", e),
            Error::Unexpected => write!(f, "UnexpectedError"),
        }
    }
}

impl From<surf::Error> for Error {
    fn from(err: surf::Error) -> Error {
        Error::NetworkError(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str { "" }
}

#[derive(Debug, Default, Clone)]
pub struct Data {
    pub url: String,
    pub images: Vec<Image>,
    pub metadata: HashMap<String, String>,
}

impl Data {
    pub fn get_card(&self, social: Social) -> Card {
       Card::new(&self, social)
    }
}

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
}

#[derive(Debug, Clone)]
pub enum CardSize {
    Small, // Mastodon
    Medium, // Twitter sumary
    Large, // Twitter summary with large image || Facebook
}

#[derive(Debug, Clone)]
pub enum Social {
    Facebook,
    Mastodon,
    Twitter,
}

#[derive(Debug, Clone)]
pub struct Card {
    pub title: String,
    pub site: String,
    pub description: Option<String>,
    pub image: Option<Image>,
    pub size: Option<CardSize>,
    pub social: Social,
}

impl Card {
    pub fn new(data: &Data, social: Social) -> Card {
        
        let metadata = data.metadata.clone();
        let mut site = data.url.clone();
        let mut size = None;

        let mut title_find = vec_of_strings!["og:title", "twitter:title", "title"];
        let mut description_find = vec_of_strings!["og:description", "twitter:description", "description"];
        let mut image_find = vec_of_strings!["og:image", "twitter:image", "twitter:image:src"];

        match social {
            Social::Facebook => {
                
            },
            Social::Mastodon => {
                image_find = vec_of_strings!["og:image"];

                if metadata.contains_key("og:site_name") {
                    site = metadata.get("og:site_name").unwrap().to_string();
                }                
            },
            Social::Twitter => {
                title_find = vec_of_strings!["twitter:title", "og:title", "title"];
                description_find = vec_of_strings!["twitter:description", "og:description", "description"];
                image_find = vec_of_strings!["twitter:image", "twitter:image:src", "og:image"];

                if metadata.contains_key("twitter:card") {
                    match metadata.get("twitter:card").unwrap().as_str() {
                        "summary_large_image" => size = Some(CardSize::Large),
                        "summary" => size = Some(CardSize::Medium),
                        _ => ()
                    }
                } else {
                    size = Some(CardSize::Medium);
                }
            }
        }

        let pre_image = Card::get_correct_tag(&image_find, &metadata);
        let title = Card::get_correct_tag(&title_find, &metadata).unwrap();
        let description = Card::get_correct_tag(&description_find, &metadata);
        let image = match pre_image {
            Some(url) => Some(Image::new(url)),
            None => (None)
        };

        Card {title, site, description, image, size, social}
    }

    pub fn get_correct_tag(
            list: &Vec<String>,
            metadata: &HashMap<String, String>) -> Option<String> {

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
