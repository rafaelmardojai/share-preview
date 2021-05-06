use once_cell::sync::Lazy;
pub mod card;
pub mod data;
pub mod image;
pub mod scraper;

// surf Client for backend requests
pub static CLIENT: Lazy<surf::Client> =
    Lazy::new(|| surf::Client::new().with(surf::middleware::Redirect::default()));

pub use self::{
    card::{Card, CardSize, Social},
    data::Data,
    image::Image,
    scraper::{scrape, Error}
};
