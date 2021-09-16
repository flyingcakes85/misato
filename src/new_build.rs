#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use comrak::{markdown_to_html, ComrakOptions};
use extract_frontmatter::Extractor;
use handlebars::Handlebars;
use rsass::{compile_scss_path, output};
use serde::Serialize;
use serde_json::value::Map;
use serde_json::Value as Json;
use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsStr,
    fmt::Result,
    fs,
    fs::create_dir_all,
    path::{Path, PathBuf},
    str,
};
use toml::Deserializer;
use toml::Value as Toml;
// use toml::Value;
use walkdir::WalkDir;

pub fn build() {
    let base_attributes = get_attributes();

    let mut handlebars = Handlebars::new();

    discover_pages(&mut handlebars);
}

fn get_attributes() -> Map<String, Json> {
    let mut attributes: Map<String, Json> = Map::new();

    let config_string = fs::read_to_string("config.toml").unwrap();
    let config_toml = config_string.parse::<toml::Value>().unwrap();

    for c in config_toml.as_table() {
        for t in c {
            attributes.insert(
                t.0.to_string(),
                serde_json::from_str(&serde_json::to_string(&toml_to_json(t.1.clone())).unwrap())
                    .unwrap(),
            );
        }
    }

    attributes
}

pub fn discover_pages(handlebars: &mut Handlebars) {
    if Path::new("pages").exists() {
        for source_path in WalkDir::new("pages") {
            let source_path = source_path.unwrap();

            if source_path.path().extension().and_then(OsStr::to_str) == Some("html")
                || source_path.path().extension().and_then(OsStr::to_str) == Some("hbs")
            {
                handlebars
                    .register_template_file(
                        &get_file_name(&source_path.path()),
                        &source_path.path(),
                    )
                    .unwrap();
            }
        }
    }
}
fn toml_to_json(toml: Toml) -> Json {
    match toml {
        Toml::String(s) => Json::String(s),
        Toml::Integer(i) => Json::Number(i.into()),
        Toml::Float(f) => {
            let n = serde_json::Number::from_f64(f).expect("float infinite and nan not allowed");
            Json::Number(n)
        }
        Toml::Boolean(b) => Json::Bool(b),
        Toml::Array(arr) => Json::Array(arr.into_iter().map(toml_to_json).collect()),
        Toml::Table(table) => Json::Object(
            table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect(),
        ),
        Toml::Datetime(dt) => Json::String(dt.to_string()),
    }
}

fn get_file_name(p: &Path) -> String {
    p.file_name()
        .unwrap()
        .to_str()
        .to_owned()
        .ok_or("[ERR] Could not get file name")
        // TODO : Handle error
        .unwrap()
        .to_string()
}
