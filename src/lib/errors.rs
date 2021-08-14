#[derive(Debug)]
pub enum GobanSVGError {
    InvalidRange,
    UnlabellableRange,
}

impl std::fmt::Display for GobanSVGError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRange => write!(f, "Invalid range to render in goban."),
            Self::UnlabellableRange => write!(f, "Range too large for use with labels."),
        }
    }
}

impl std::error::Error for GobanSVGError {}
