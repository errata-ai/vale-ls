use std::collections::HashMap;

use reqwest;
use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct Regex101Session {
    pub version_delete_code: String,
    pub regex_delete_code: String,
    pub permalink_fragment: String,
    pub version: i32,
    pub is_library_entry: bool,
}

pub(crate) fn upload(pattern: String) -> Result<Regex101Session, Error> {
    let mut map = HashMap::new();

    map.insert("regex", pattern.as_str());
    map.insert("flags", "gm");
    map.insert("testString", "Enter your test content here.");
    map.insert("flavor", "pcre2");
    map.insert("delimiter", "/");

    let resp = reqwest::blocking::Client::new()
        .post("https://regex101.com/api/regex")
        .json(&map)
        .send()?;

    let body = resp.text()?;
    let session: Regex101Session = serde_json::from_str(&body)?;

    Ok(session)
}
