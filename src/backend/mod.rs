use once_cell::sync::Lazy;
pub mod card;
pub mod data;
pub mod image;
pub mod log;
pub mod scraper;
pub mod social;

// surf Client for backend requests
pub static CLIENT: Lazy<surf::Client> =
    Lazy::new(|| surf::Client::new().with(surf::middleware::Redirect::default()));

#[macro_export]
macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

pub use self::{
    card::{Card, CardError, CardSize},
    data::{Meta, Data},
    image::{Image, ImageError},
    log::{Log, LogLevel},
    scraper::{scrape, Error},
    social::{Social, SocialConstraints, SocialImageSizeKind},
};
