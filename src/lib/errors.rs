use super::goban::GobanError;
use sgf_parse::SgfParseError;

#[derive(Debug)]
pub enum GobanSVGError {
    GobanError(GobanError),
    ParseError(SgfParseError),
    InvalidRange,
    UnlabellableRange,
}

impl std::fmt::Display for GobanSVGError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GobanError(e) => write!(f, "{}", e),
            Self::ParseError(e) => write!(f, "{}", e),
            Self::InvalidRange => write!(f, "Invalid range to render in goban."),
            Self::UnlabellableRange => write!(f, "Range too large for use with labels."),
        }
    }
}

impl std::error::Error for GobanSVGError {}

impl From<SgfParseError> for GobanSVGError {
    fn from(error: SgfParseError) -> Self {
        Self::ParseError(error)
    }
}

impl From<GobanError> for GobanSVGError {
    fn from(error: GobanError) -> Self {
        Self::GobanError(error)
    }
}
