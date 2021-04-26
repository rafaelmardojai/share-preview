use std::fmt::{Display, Formatter, Result as FmtResult};
use std::error;

use surf;
use url::{Url, ParseError};

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

#[derive(Debug, Default)]
pub struct Metadata {
    pub title: String,
    pub url: String,
    pub images: Vec<Image>,
    pub description: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<Image>,
    pub tw_title: Option<String>,
    pub tw_description: Option<String>,
    pub tw_image: Option<Image>,
}

impl Metadata {
    pub fn new(meta: &Vec<(String, String)>) -> Metadata {
        let mut result = Metadata::default();
        
        // Select the meta tags that we want
        for prop in meta.iter() {
            let key: &str = &(prop.0);
            let val = prop.1.clone();
            match key {
                "title" => { result.title = val; },
                "description" => { result.description = Some(val); },
                "og:title" => { result.og_title = Some(val); },
                "og:description" => { result.og_description = Some(val); },
                "og:image" => { result.og_image = Some(Image::new(val)); },
                "twitter:title" => { result.tw_title = Some(val); },
                "twitter:description" => { result.tw_description = Some(val); },
                "twitter:image" => { result.tw_image = Some(Image::new(val)); },
                "twitter:image:src" => { result.tw_image = Some(Image::new(val)); },
                _ => {},
            }
        }

        result
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
            println!("{:?}", e);
        }
        self
    }
}