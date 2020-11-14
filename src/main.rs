mod lib;

use lib::{Goban, MakeSvgOptions};
use std::path::Path;

fn main() {
    // TODO: Parse these from arguments.
    let infile = Path::new("/home/julian/Downloads/tsumego/prob0001.sgf");
    let outfile = Path::new("/tmp/out.png");
    let options: MakeSvgOptions = Default::default();
    let move_number = 1;

    let goban = match load_goban(&infile, move_number) {
        Ok(goban) => goban,
        Err(e) => {
            eprintln!("Failed to load requested node from SGF: {}", e);
            std::process::exit(1);
        }
    };

    let document = match lib::make_svg(&goban, options) {
        Ok(document) => document,
        Err(e) => {
            eprintln!("Failed to generate SVG: {}", e);
            std::process::exit(1);
        }
    };

    match outfile.extension().and_then(std::ffi::OsStr::to_str) {
        Some("svg") => {
            if let Err(e) = svg::save(&outfile, &document) {
                eprintln!("Failed to save svg output: {}", e);
                std::process::exit(1);
            }
        }
        Some("png") => {
            #[cfg(not(feature = "png"))]
            {
                eprintln!("'png' feature not enabled.");
                std::process::exit(1);
            }
            #[cfg(feature = "png")]
            if let Err(e) = save_png(&outfile, &document) {
                eprintln!("Failed to save svg output: {}", e);
                std::process::exit(1);
            }
        }
        extension => {
            eprintln!("Unsupported file extension: '{}'", extension.unwrap_or(""));
            std::process::exit(1);
        }
    }
}

fn load_goban<P: AsRef<Path>>(
    filename: &P,
    move_number: u64,
) -> Result<Goban, Box<dyn std::error::Error>> {
    let mut sgf_node = &get_sgf_root(filename)?;

    let mut goban = Goban::from_sgf_node(&sgf_node)?;
    for _ in 1..move_number {
        sgf_node = sgf_node
            .children()
            .next()
            .ok_or(SgfRenderError::InsufficientSgfNodes)?;
        goban.process_node(sgf_node)?;
    }

    Ok(goban)
}

fn get_sgf_root<P: AsRef<Path>>(
    filename: &P,
) -> Result<sgf_parse::SgfNode, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(filename)?;
    let collection = sgf_parse::parse(&text)?;
    collection
        .into_iter()
        .next()
        .ok_or(Box::new(SgfRenderError::NoSgfNodes))
}

#[cfg(feature = "png")]
fn save_png<P: AsRef<Path>>(
    filename: &P,
    document: &svg::node::element::SVG,
) -> Result<(), Box<dyn std::error::Error>> {
    let s = document.to_string();
    let mut fontdb = usvg::fontdb::Database::new();
    let font_data = include_bytes!("../data/Roboto-Bold.ttf").to_vec();
    fontdb.load_font_data(font_data);
    let tree = usvg::Tree::from_str(
        &s,
        &usvg::Options {
            fontdb,
            ..Default::default()
        },
    )?;
    let img =
        resvg::render(&tree, usvg::FitTo::Original, None).ok_or(SgfRenderError::PNGRenderFailed)?;
    img.save_png(filename)?;
    Ok(())
}

#[derive(Debug)]
enum SgfRenderError {
    NoSgfNodes,
    InsufficientSgfNodes,
    PNGRenderFailed,
}

impl std::fmt::Display for SgfRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSgfNodes => write!(f, "No sgf nodes found in collection."),
            Self::InsufficientSgfNodes => {
                write!(f, "Insufficient SGF nodes for requested move number.")
            }
            Self::PNGRenderFailed => write!(f, "Failed to render png from svg."),
        }
    }
}

impl std::error::Error for SgfRenderError {}
