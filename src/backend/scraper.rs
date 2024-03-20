// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    error,
    fmt::{Display, Formatter, Result as FmtResult}
};

use data_url::DataUrl;
use url::{Url, ParseError};
use scraper::{Html, Selector, element_ref::ElementRef};

use super::{Data, Meta, Image, CLIENT};

const IMAGE_TAGS: [&str; 3] = ["og:image", "twitter:image", "twitter:image:src"];

pub async fn scrape(url: &Url) -> Result<Data, Error> {
    //! Request URL html body and scrape it to get the needed data

    let mut resp = CLIENT.get(&url).await?;

    if resp.status().is_success() {
        let mut data = Data::default();

        // Store favicon urls
        let mut html_icons: Vec<String> = Vec::default();

        // Call function to get data from html:
        get_html_data(&resp.body_string().await?, &mut data, &url, &mut html_icons).await; // Write html data to a Vec<>

        data.favicon = get_favicon(&url, html_icons).await;  // Set data favicon
        data.url = url.host_str().unwrap().to_string(); // Set Data URL

        Ok(data)
    } else {
        Err(Error::Unexpected(resp.status().to_string()))
    }
}

async fn get_html_data(
        text: &String,
        data: &mut Data,
        url: &Url,
        icons: &mut Vec<String>) {
    //! Parse html and get data

    let document = Html::parse_document(&text); // HTML document from request text

    // Get document title
    let selector = Selector::parse("title").unwrap(); // HTML <title> selector
    // Try to get document title
    if let Some(title) = document.select(&selector).next() {
        data.title = Some(title.inner_html().trim().to_string());
    }

    // Get meta tags
    let selector = Selector::parse("meta").unwrap();
    for element in document.select(&selector) {
        let name: Option<String> = get_attr_val(&element, "name");
        let pre_property: Option<String> = get_attr_val(&element, "property");
        let property: Vec<String> = match pre_property {
            Some(value) => value.split(" ").map(|s| s.to_string()).collect(),
            None => Vec::new()
        };
        let content: Option<String> = get_attr_val(&element, "content");
        let image: Option<Image> = match (is_image(&name, &property), &content) {
            (true, Some(val)) => {
                let mut image = Image::new(val.to_string());
                image.normalize(url);
                Some(image)
            },
            _ => None
        };

        if let (Some(_), _) | (_, Some(_)) = (&name, property.last()) {
            let meta = Meta {name, property, content, image };
            data.metadata.push(meta);
        }
    }

    // Get icons
    let selector = Selector::parse("link").unwrap();
    for element in document.select(&selector) {
        let rel: Option<String> = get_attr_val(&element, "rel");
        let href: Option<String> = get_attr_val(&element, "href");

        if let Some(value) = rel {
            match value.as_str() {
                "shortcut icon" | "icon" => {
                    if let Some(url) = href {
                        icons.push(url);
                    }
                },
                _ => {},
            }
        }
    }

    // Get images
    let selector = Selector::parse("img").unwrap();
    for element in document.select(&selector) {
        if let Some(src) = element.value().attr("src") {
            let src = src.trim().to_string();
            if src.contains(".jpg") || src.contains(".jpeg") || src.contains(".png"){
                let mut image = Image::new(src);
                image.normalize(url);
                data.body_images.push(image)
            }
        }
    }
}

fn get_attr_val(element: &ElementRef, attr: &str) -> Option<String> {
    element.value().attr(attr).and_then(|val| {
        let value = val.to_string().trim().to_string().replace('\n', " ");
        Some(value)
    })
}

fn is_image(name: &Option<String>, property: &Vec<String>) -> bool {
    for term in IMAGE_TAGS.iter() {
        if property.contains(&term.to_string()) {
            return true;
        } else if let Some(s) = name {
            if &term.to_string() == s {
                return true;
            }
        }
    }
    false
}

async fn get_favicon(url: &Url, html_icons: Vec<String>) -> Option<Image> {

    // Filter valid urls and fix relative paths
    let mut icons: Vec<Url> = html_icons.iter().filter_map(|rel| {
        match Url::parse(&rel.as_str()) {
            Ok(icon_url) => {
                Some(icon_url)
            },
            Err(err) => {
                match err {
                    ParseError::RelativeUrlWithoutBase => {
                        if let Ok(icon_url) = url.join(&rel) {
                            Some(icon_url)
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            }
        }
    }).collect();

    if let Ok(favicon) = Url::parse(url.origin().unicode_serialization().as_str()) {
        if let Ok(favicon) = favicon.join("favicon.ico") {
            icons.push(favicon);
        }
    }

    for icon in icons.iter() {
        match icon.scheme() {
            "file" | "http" | "https" => {
                if let Ok(mut resp) = CLIENT.get(&icon).await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.body_bytes().await {
                            let image = Image::new(icon.to_string());
                            image.size.set(Some(bytes.len()));
                            image.bytes.replace(Some(bytes));
                            return Some(image);
                        };
                    }
                }
            }
            "data" => {
                if let Ok(data) = DataUrl::process(icon.as_str()) {
                    if let Ok((body, _fragment))  = data.decode_to_vec() {
                        let image = Image::new(icon.to_string());
                        image.size.set(Some(body.len()));
                        image.bytes.replace(Some(body));
                        return Some(image);
                    }
                }
            }
            _ => ()
        }
    }

    None
}

#[derive(Debug)]
pub enum Error {
    NetworkError(surf::Error),
    Unexpected(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Error::NetworkError(ref e) => write!(f, "NetworkError:  {}", e),
            Error::Unexpected(ref status) => write!(f, "UnexpectedError: Error {}", status),
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
