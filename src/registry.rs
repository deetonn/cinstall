// GOAL: This should be a full registry containing common packages
//       so that the user can simply use `cinstall fmt` for example.
//
// This only really needs a map of a simple name to the URL.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub enum Language {
    CXX,
    C,
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Language::CXX => "C++".into(),
            Language::C => "C".into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub url: &'static str,
    // simple description for that package.
    pub description: &'static str,
    // which language is used
    pub language: Language,
}

impl Package {
    pub fn get_url(&self) -> &'static str {
        self.url
    }
    pub fn get_description(&self) -> &'static str {
        self.description
    }
    pub fn get_language(&self) -> &Language {
        &self.language
    }
}

impl Package {
    pub fn new(url: &'static str, desc: &'static str, lang: Language) -> Self {
        Self {
            url,
            description: desc,
            language: lang,
        }
    }
}

pub struct PackageRegistry {
    reg: HashMap<&'static str, Package>,
}

impl Default for PackageRegistry {
    fn default() -> Self {
        let json = include_str!("pkg_reg.json");
        let map = match serde_json::from_str::<HashMap<&'static str, Package>>(json) {
            Ok(m) => m,
            Err(e) => panic!("failed to deserialize registry json: {}", e),
        };

        Self { reg: map }
    }
}

impl PackageRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: &str) -> Option<&Package> {
        self.reg.get(id)
    }

    pub fn packages(&self) -> &HashMap<&'static str, Package> {
        &self.reg
    }
}
