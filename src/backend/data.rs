// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use super::{Card, CardError, Image, Social};

#[derive(Debug, Default, Clone)]
pub struct Data {
    pub url: String,
    pub title: Option<String>,
    pub images: Vec<Image>,
    pub metadata: HashMap<String, String>,
}

impl Data {
    pub fn get_card(&self, social: Social) -> Result<Card, CardError> {
       Card::new(&self, social)
    }

    pub fn get_metadata(&self) -> HashMap<String, String> {
        self.metadata.iter().map(|(k,v)| {
            let k = k.clone();
            let v = v.clone();
            (k, v)
        }).collect::<HashMap<String, String>>()
    }
}
