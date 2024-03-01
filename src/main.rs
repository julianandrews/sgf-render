use std::error::Error;
use std::path::Path;

use clap::Parser;
use minidom::Element;

fn main() {
    let parsed_args = sgf_render::SgfRenderArgs::parse();
    let options = match parsed_args.make_svg_args.options() {
        Ok(options) => options,
        Err(e) => {
            eprintln!("Failed to read input: {}", e);
            std::process::exit(1);
        }
    };

    let input = match read_input(parsed_args.infile) {
        Ok(goban) => goban,
        Err(e) => {
            eprintln!("Failed to read input: {}", e);
            std::process::exit(1);
        }
    };

    let svg = match sgf_render::make_svg(&input, &options) {
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

fn read_input<P: AsRef<Path>>(infile: Option<P>) -> Result<String, Box<dyn Error>> {
    let mut reader: Box<dyn std::io::Read> = match infile {
        Some(filename) => Box::new(std::io::BufReader::new(std::fs::File::open(&filename)?)),
        None => Box::new(std::io::stdin()),
    };
    let mut input = String::new();
    reader.read_to_string(&mut input)?;
    Ok(input)
}

fn write_output<P: AsRef<Path>>(svg: &Element, outfile: Option<P>) -> Result<(), Box<dyn Error>> {
    match outfile {
        Some(filename) => write_to_file(filename.as_ref(), svg)?,
        None => svg.write_to(&mut std::io::stdout())?,
    }
    Ok(())
}

fn write_to_file(outfile: &Path, svg: &Element) -> Result<(), Box<dyn Error>> {
    match outfile.extension().and_then(std::ffi::OsStr::to_str) {
        Some("svg") => {
            let mut file = std::fs::File::create(outfile)?;
            svg.write_to(&mut file)?;
        }
        Some("png") => save_png(outfile, svg)?,
        _ => return Err(SgfRenderError::UnsupportedFileExtension.into()),
    }
    Ok(())
}

#[cfg(feature = "png")]
fn save_png(outfile: &Path, svg: &Element) -> Result<(), Box<dyn Error>> {
    let mut buffer: Vec<u8> = vec![];
    svg.write_to(&mut buffer)?;
    let s = std::str::from_utf8(&buffer)?;
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_font_data(include_bytes!("../resources/Inter-Bold.ttf").to_vec());
    let tree = usvg::Tree::from_str(
        s,
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
fn save_png(_outfile: &Path, _document: &Element) -> Result<(), Box<dyn Error>> {
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

impl std::error::Error for SgfRenderError {}
