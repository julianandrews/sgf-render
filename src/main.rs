use minidom::Element;
use std::error::Error;
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");

use sgf_render::args;

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
    if parsed_args.print_version {
        println!("sgf-render {}", VERSION);
        return;
    }

    let input = match read_input(parsed_args.infile) {
        Ok(goban) => goban,
        Err(e) => {
            eprintln!("Failed to read input: {}", e);
            std::process::exit(1);
        }
    };

    let svg = match sgf_render::make_svg(&input, &parsed_args.options) {
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
    let font_data = include_bytes!("../resources/Roboto-Bold.ttf").to_vec();
    fontdb.load_font_data(font_data);
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
