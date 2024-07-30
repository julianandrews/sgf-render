mod generated_styles;
mod goban_range;
mod goban_style;
mod options;

pub mod svg;
pub mod text;

pub use generated_styles::GeneratedStyle;
pub use goban_range::GobanRange;
pub use goban_style::GobanStyle;
pub use options::{BoardSideSet, MoveNumberOptions, NodeDescription, NodeNumber, RenderOptions};

fn board_label_text(x: u8) -> String {
    if x + b'A' < b'I' {
        ((x + b'A') as char).to_string()
    } else {
        ((x + b'B') as char).to_string() // skip 'I'
    }
}
