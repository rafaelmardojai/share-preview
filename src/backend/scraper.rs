use std::collections::HashMap;

use surf;
use url::Url;
use scraper::{Html, Selector};
use scraper::element_ref::ElementRef;

use super::elements::{Data, Image, Error};

pub async fn scrape(url: &Url) -> Result<Data, Error> {
    let mut resp = surf::get(&url).await?;

    if resp.status().is_success() {
        let mut metadata = HashMap::new(); // Empty HashMap<String, String> to store meta tags
        let mut images = Vec::new(); // Empty Vec<Image> to store images from HTML

        // Call function to get data from request text:
        get_html_data(&resp.body_string().await?, &mut metadata, &mut images).await; // Write html data to a Vec<>

        let mut data = Data::default();
        data.url = url.host_str().unwrap().to_string(); // Set Metadata URL
        data.metadata = metadata;
        // Collect a new Images Vec<> with the local URLs normalized:
        data.images = images.iter().map(|i| {
            let mut i = i.clone();
            i.normalize(&url);
            i
        }).collect::<Vec<Image>>();

        Ok(data) // Return Metadata
    } else {
        Err(Error::Unexpected)
    }
}

async fn get_html_data(text: &String,
                 meta: &mut HashMap<String, String>,
                 images: &mut Vec<Image>) {

    let document = Html::parse_document(&text); // HTML document from request text

    // Get document title
    let selector = Selector::parse("title").unwrap(); // HTML <title> selector
    let title = document.select(&selector).next().unwrap();
    // Push title to the meta Vec<>
    meta.insert(String::from("title"), title.inner_html().trim().to_string());

    // Get meta tags
    let selector = Selector::parse("meta").unwrap();
    for element in document.select(&selector) {
        match get_meta_prop(&element, "property") {
            Some((key, content)) => {
                let mut content = content.trim().to_string();
                content = content.replace('\n', " ");
                meta.insert(key, content);
            }
            None => (),
        }
        match get_meta_prop(&element, "name") {
            Some((key, content)) => {
                let mut content = content.trim().to_string();
                content = content.replace('\n', " ");
                meta.insert(key, content);
            },
            None => (),
        }
    }

    // Get images
    let selector = Selector::parse("img").unwrap();
    for element in document.select(&selector) {
        match element.value().attr("src") {
            Some(src) => {
                let src = src.trim().to_string();
                if src.contains(".jpg") || src.contains(".jpeg") || src.contains(".png"){
                    images.push(Image::new(src))
                }
            },
            None => (),
        }
    }
}

fn get_meta_prop(element: &ElementRef, name: &str) -> Option<(String, String)> {
    element.value().attr(name).and_then(|key|
        element.value().attr("content").map(|content| (key.to_string(), content.to_string())))
}
