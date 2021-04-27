// Copyright 2021 Rafael Mardojai CM
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use super::{Card, Image, Social};

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