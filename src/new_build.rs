// #![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use comrak::{markdown_to_html, plugins, ComrakOptions};
use extract_frontmatter::Extractor;
use handlebars::{to_json, Handlebars};
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
    process::exit,
    str,
};
use toml::Value as Toml;
use toml::{to_vec, Deserializer};
// use toml::Value;
use walkdir::WalkDir;

pub fn build() {
    fs::create_dir_all("target").unwrap();
    let mut base_attributes: Map<String, Json> = get_attributes();

    let mut handlebars = Handlebars::new();

    let mut renderlist_pages = Vec::<(String, PathBuf)>::new();
    let mut renderlist_posts = Vec::<(String, PathBuf)>::new();

    discover_pages(&mut handlebars, &mut renderlist_pages);
    discover_posts(&mut handlebars, &mut renderlist_posts);

    discover_layouts(&mut handlebars);

    let post_list = render_posts(renderlist_posts, &handlebars, &base_attributes);

    base_attributes.insert("posts".to_string(), toml_to_json(Toml::Array(post_list)));
    // base_attributes.insert("categories".to_string(), serde_json::fropost_list);
    // let s = String::from_utf8(serde_json::ser::to_vec_pretty(&post_list).unwrap()).unwrap();
    // println!("{}", s);
    // println!("post list is {:?}", post_list);

    // println!("{:?}", post_list);

    // base_attributes.insert(
    //     "posts".to_string(),
    //     serde_json::from_str(
    //         &String::from_utf8(serde_json::ser::to_vec(&post_list).unwrap()).unwrap(),
    //     )
    //     .unwrap(),
    // );

    // println!("{:?}", base_attributes);

    render_pages(renderlist_pages, &handlebars, &base_attributes);
    generate_css();

    // println!("{:#?}", base_attributes);
}

fn render_pages(
    renderlist: Vec<(String, PathBuf)>,
    handlebars: &Handlebars,
    data: &Map<String, Json>,
) {
    for (template_name, template_path) in renderlist {
        let dest_path = template_path.to_str().unwrap().replace("pages/", "target/");

        fs::write(
            Path::new(&dest_path).with_extension("html"),
            handlebars.render(&template_name, &data).unwrap(),
        )
        .unwrap();
    }
}

// render posts and return a vector of posts
fn render_posts(
    renderlist: Vec<(String, PathBuf)>,
    handlebars: &Handlebars,
    data: &Map<String, Json>,
) -> Vec<Toml> {
    let mut html_to_write: String;
    let mut post_list: Vec<Toml> = Vec::<Toml>::new();
    let comark_options = ComrakOptions::default();
    let mut dest_path: PathBuf;
    let mut rel_path: PathBuf;

    for (template_name, template_path) in renderlist {
        // read the source md text
        let source_md_data = fs::read_to_string(&template_path).unwrap();

        // split source_md_data to front matter and actual text
        // front_matter is Toml
        let (mut front_matter, raw_md_text) = parse_front_matter(source_md_data);

        let raw_md_text = raw_md_text.trim().to_string();

        // plug data has all attributes
        let mut plug_data: Map<String, Json> = data.clone();

        // rel_path = PathBuf::new();
        dest_path = PathBuf::new();
        dest_path.push("target");

        // create a local clone of handlebars
        let mut md_handlebars = handlebars.clone();

        // println!("froh {}", rel_path.to_str().unwrap());
        // front_matter["data"].as_table_mut().unwrap().clone().insert(
        //     "rel_path".to_string(),
        //     Toml::String(String::from(rel_path.to_str().unwrap())),
        // );

        // println!("{:#?}\n", front_matter);

        // TODO : this generates a relative path
        // configure it to use complete domain path
        rel_path = get_dest_path(&plug_data, &mut md_handlebars);
        rel_path.push(template_name);
        rel_path.set_extension("html");
        dest_path.push(rel_path.clone());

        for (k, v) in front_matter.as_table().unwrap() {
            plug_data.insert(k.to_string(), value_to_json(v));
        }

        front_matter["data"].as_table_mut().unwrap().insert(
            "rel_path".to_string(),
            Toml::String(String::from(rel_path.to_str().unwrap())),
        );

        // register the raw markdown data
        md_handlebars
            .register_template_string("raw_markdown", raw_md_text)
            .unwrap();

        // render markdown with attributes
        let plugged_md_text = md_handlebars.render("raw_markdown", &plug_data).unwrap();

        // convert plugged markdown to html
        let generated_html = markdown_to_html(&plugged_md_text, &comark_options);

        // register the final markdown_data template
        // at this point this it technically html
        // but registering it as markdown_data makes
        // sense for end user
        md_handlebars
            .register_template_string("markdown_data", &generated_html)
            .unwrap();

        // generate final html
        // and set rel_path
        html_to_write = if json_map_key_exists(&plug_data, "data", "layout") {
            md_handlebars
                .render(front_matter["data"]["layout"].as_str().unwrap(), &plug_data)
                .unwrap()
        } else {
            generated_html
        };

        println!("dest path we got : {:#?}", dest_path);

        post_list.push(front_matter.clone());

        println!("end of day file to write {:#?}", dest_path);

        // write out the html file
        fs::create_dir_all(&dest_path.parent().unwrap()).unwrap();
        fs::write(&dest_path, &html_to_write).unwrap();
    }

    post_list
}

fn value_to_json(toml: &Toml) -> Json {
    serde_json::from_str(&serde_json::to_string_pretty(&toml_to_json(toml.clone())).unwrap())
        .unwrap()
}

fn parse_front_matter(source_data: String) -> (Toml, String) {
    let mut extractor = Extractor::new(&source_data);
    extractor.select_by_terminator("+++");
    extractor.discard_first_line();

    let (front_matter, document): (Vec<&str>, &str) = extractor.split();

    let document = document.trim().to_string();
    let front_matter = front_matter.join("\n");

    let front_matter_toml = front_matter.parse::<Toml>().unwrap();

    (front_matter_toml, document)
}

// checks if keys exists in 2 level json map
fn json_map_key_exists(data: &Map<String, Json>, k1: &str, k2: &str) -> bool {
    if data.contains_key(k1) {
        if data[k1].as_object().unwrap().contains_key(k2) {
            return true;
        }
        return false;
    }
    false
}

fn get_dest_path(data: &Map<String, Json>, handlebars: &mut Handlebars) -> PathBuf {
    let mut dest_path = PathBuf::new();

    if json_map_key_exists(&data, "data", "path") {
        println!("found custom path in post");
        for p in data["data"]["path"].as_str().unwrap().split("/") {
            dest_path.push(p.to_string());
        }
    } else if json_map_key_exists(&data, "config", "blog_path") {
        println!("Using path defined in global config");
        handlebars
            .register_template_string(
                "blog_path_internal",
                data["config"]["blog_path"].as_str().unwrap(),
            )
            .unwrap();

        for p in handlebars
            .render("blog_path_internal", &data)
            .unwrap()
            .split("/")
        {
            dest_path.push(p.to_string());
        }
    } else {
        println!("[ERR]No post path found");
        // TODO : Don't make app exit when no path found;
        // use a generic path in this case
        exit(0);
    }

    dest_path
}

fn get_attributes() -> Map<String, Json> {
    let mut attributes: Map<String, Json> = Map::new();

    let config_string = fs::read_to_string("config.toml").unwrap();
    let config_toml = config_string.parse::<toml::Value>().unwrap();

    for c in config_toml.as_table() {
        for t in c {
            attributes.insert(
                t.0.to_string(),
                serde_json::from_str(
                    &serde_json::to_string_pretty(&toml_to_json(t.1.clone())).unwrap(),
                )
                .unwrap(),
            );
        }
    }

    attributes
}

pub fn discover_pages(handlebars: &mut Handlebars, renderlist: &mut Vec<(String, PathBuf)>) {
    if Path::new("pages").exists() {
        for source_path in WalkDir::new("pages") {
            let source_path = source_path.unwrap();

            if source_path.path().extension().and_then(OsStr::to_str) == Some("html")
                || source_path.path().extension().and_then(OsStr::to_str) == Some("hbs")
            {
                let mut template_name = get_file_name(&source_path.path());
                template_name.push_str("_page");
                renderlist.push((template_name.clone(), source_path.path().to_path_buf()));
                handlebars
                    .register_template_file(&template_name, &source_path.path())
                    .unwrap();
            }
        }
    }
}

pub fn discover_posts(handlebars: &mut Handlebars, renderlist: &mut Vec<(String, PathBuf)>) {
    if Path::new("posts").exists() {
        for source_path in WalkDir::new("posts") {
            let source_path = source_path.unwrap();

            if source_path.path().extension().and_then(OsStr::to_str) == Some("md") {
                let mut template_name = get_file_name(&source_path.path());
                template_name.push_str("_post");
                renderlist.push((template_name.clone(), source_path.path().to_path_buf()));
                handlebars
                    .register_template_file(&template_name, &source_path.path())
                    .unwrap();
            }
        }
    }
}

pub fn discover_layouts(handlebars: &mut Handlebars) {
    if Path::new("layouts").exists() {
        for source_path in WalkDir::new("layouts") {
            let source_path = source_path.unwrap();

            if source_path.path().extension().and_then(OsStr::to_str) == Some("html")
                || source_path.path().extension().and_then(OsStr::to_str) == Some("hbs")
            {
                let mut template_name = get_file_name(&source_path.path());
                template_name.push_str("_layout");
                handlebars
                    .register_template_file(&template_name, &source_path.path())
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
    Path::new(p.file_stem().unwrap())
        .file_name()
        .unwrap()
        .to_str()
        .to_owned()
        .ok_or("[ERR] Could not get file name")
        // TODO : Handle error
        .unwrap()
        .to_string()
}

fn generate_css() {
    for p in WalkDir::new("styles") {
        let p = p.unwrap();
        let p = p.path();
        if p.extension().and_then(OsStr::to_str) == Some("css") {
            fs::copy(p, p.to_str().unwrap().replace("styles/", "target/")).unwrap();
        }
    }
}
