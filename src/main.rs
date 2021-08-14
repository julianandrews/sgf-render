mod args;
mod lib;

use std::error::Error;
use std::path::Path;
use svg::node::element::SVG;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let opts = args::build_opts();
    let parsed_args = match args::parse(&opts, &args) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error);
            args::print_usage(&args[0], &opts);
            std::process::exit(1);
        }
    };
    if parsed_args.print_help {
        args::print_usage(&args[0], &opts);
        return;
    }

    let goban = match load_goban(parsed_args.infile, parsed_args.node_description) {
        Ok(goban) => goban,
        Err(e) => {
            eprintln!("Failed to load SGF node: {}", e);
            std::process::exit(1);
        }
    };

    let svg = match lib::make_svg(&goban, &parsed_args.options) {
        Ok(svg) => svg,
        Err(e) => {
            eprintln!("Failed to generate SVG: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = write_output(&svg, parsed_args.outfile) {
        eprintln!("Failed to write output: {}", e);
        std::process::exit(1);
    }
}

fn load_goban<P: AsRef<Path>>(
    infile: Option<P>,
    node_description: lib::NodeDescription,
) -> Result<lib::Goban, Box<dyn Error>> {
    let mut reader: Box<dyn std::io::Read> = match infile {
        Some(filename) => Box::new(std::io::BufReader::new(std::fs::File::open(&filename)?)),
        None => Box::new(std::io::stdin()),
    };
    let mut text = String::new();
    reader.read_to_string(&mut text)?;
    let collection = sgf_parse::go::parse(&text)?;
    lib::Goban::from_node_in_collection(node_description, &collection)
}

fn write_output<P: AsRef<Path>>(svg: &SVG, outfile: Option<P>) -> Result<(), Box<dyn Error>> {
    match outfile {
        Some(filename) => write_to_file(filename.as_ref(), svg),
        None => svg::write(std::io::stdout(), svg).map_err(|e| e.into()),
    }
}

fn write_to_file(outfile: &Path, document: &SVG) -> Result<(), Box<dyn Error>> {
    match outfile.extension().and_then(std::ffi::OsStr::to_str) {
        Some("svg") => svg::save(outfile, document)?,
        Some("png") => save_png(outfile, document)?,
        _ => return Err(SgfRenderError::UnsupportedFileExtension.into()),
    }
    Ok(())
}

#[cfg(feature = "png")]
fn save_png(outfile: &Path, document: &SVG) -> Result<(), Box<dyn Error>> {
    let s = document.to_string();
    let mut fontdb = usvg::fontdb::Database::new();
    let font_data = include_bytes!("../data/Roboto-Bold.ttf").to_vec();
    fontdb.load_font_data(font_data);
    let tree = usvg::Tree::from_str(
        &s,
        &usvg::Options {
            fontdb,
            ..usvg::Options::default()
        },
    )?;
    let pixmap_size = tree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&tree, usvg::FitTo::Original, pixmap.as_mut())
        .ok_or(SgfRenderError::PNGRenderFailed)?;
    pixmap.save_png(outfile)?;
    Ok(())
}

#[cfg(not(feature = "png"))]
fn save_png(_outfile: &Path, _document: &SVG) -> Result<(), Box<dyn Error>> {
    Err(SgfRenderError::NoPngSupport.into())
}

#[derive(Debug)]
enum SgfRenderError {
    UnsupportedFileExtension,
    #[cfg(feature = "png")]
    PNGRenderFailed,
    #[cfg(not(feature = "png"))]
    NoPngSupport,
}

impl std::fmt::Display for SgfRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFileExtension => write!(f, "Unsupported file extension."),
            #[cfg(feature = "png")]
            Self::PNGRenderFailed => write!(f, "Rendering png failed."),
            #[cfg(not(feature = "png"))]
            Self::NoPngSupport => write!(f, "Compiled without png support."),
        }
    }
}

impl Error for SgfRenderError {}
