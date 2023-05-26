use clap::Parser;
use log::{info, warn, error};
use std::{
    error::Error,
    fmt::Debug,
    fs::{self, File},
    path::Path,
};
use xml::EmitterConfig;
use xmltree::Element;

#[derive(Parser, Debug)]
struct Args {
    /// Path to search phone config XML files
    #[arg(short, long)]
    xml_path: String,
    /// Perform changes (this tool runs in dry mode by default)
    #[arg(short, long, default_value = "false")]
    confirm: bool
}

fn inject_attribute<P: AsRef<Path> + Debug>(path: P) -> Result<(), Box<dyn Error>> {
    info!("trying to open file: {:?}", path);
    let xml_content = fs::read_to_string(&path)?;
    let mut root = Element::parse(xml_content.as_bytes()).expect("Failed to parse XML");

    let attr_5g = root
        .children
        .iter()
        .enumerate()
        .find_map(|(idx, attr)| match attr {
            xmltree::XMLNode::Element(p) => {
                if let Some(value) = p.attributes.get("name") {
                    return value.eq("vonr_enabled_bool").then_some(idx);
                }
                None
            }
            _ => None,
        });

    let elem = match attr_5g {
        Some(idx) => {
            info!("found existing 5g attribute, setting to true.");
            let element = root.children.remove(idx);
            let mut el = element.as_element().expect("couldn't unwrap as element!").to_owned();
            el.attributes.insert("value".into(), "true".into());
            el
        }
        _ => {
            // missing attribute
            info!("5G attribute is missing, injecting");
            let mut new_elem = Element::new("boolean");
            new_elem.attributes.insert("value".into(), "true".into());
            new_elem
                .attributes
                .insert("name".into(), "vonr_enabled_bool".into());
            new_elem
        }
    };
    let config = EmitterConfig::new()
        .write_document_declaration(false)
        .perform_indent(true);
    root.children.push(xmltree::XMLNode::Element(elem));
    let buffer = File::create(&path)?;
    root.write_with_config(buffer, config)?;
    info!("Successfully saved {:?}", &path);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let args = Args::parse();
    let path_xml = format!("{}*.xml", args.xml_path);
    let glob = glob::glob(&path_xml)?;
    info!("Searching files in glob path: {:?}", path_xml);
    if !args.confirm{
        warn!("Dry-run mode. To apply changes, use `--confirm`.");
    }
    let mut paths_found = 0;
    for path in glob {
        paths_found += 1;
        let path = path?;
        if args.confirm{
            inject_attribute(path)?;
        }else{
            info!("Found potential target: {:?}", path.file_name());
        }
    }

    if paths_found == 0{
        error!("[E] Found 0 xml files. Make sure the path {:?} is correct.", args.xml_path);
    }
    Ok(())
}
