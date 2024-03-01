use sgf_parse::SgfParseError;

#[derive(Debug)]
pub enum MakeSvgError {
    ParseError(SgfParseError),
    StyleDefError(minidom::Error),
    InsufficientSgfNodes,
    MissingVariation,
    InvalidMoveError,
    InvalidRange,
    UnlabellableRange,
}

impl std::fmt::Display for MakeSvgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(e) => write!(f, "{}", e),
            Self::StyleDefError(e) => write!(f, "Invalid defs in style: {}", e),
            Self::InvalidMoveError => write!(f, "Invalid move"),
            Self::InsufficientSgfNodes => write!(f, "Insufficient SGF nodes found"),
            Self::MissingVariation => write!(f, "Selected variation not found."),
            Self::InvalidRange => write!(f, "Invalid range to render in goban."),
            Self::UnlabellableRange => write!(f, "Range too large for use with labels."),
        }
    }
}

impl std::error::Error for MakeSvgError {}

impl From<SgfParseError> for MakeSvgError {
    fn from(error: SgfParseError) -> Self {
        Self::ParseError(error)
    }
}

#[derive(Debug)]
pub enum UsageError {
    InvalidRange,
    StyleReadError(Box<dyn std::error::Error>),
    InvalidFirstMoveNumber,
    InvalidLastMoveNumber,
    InvalidBoardSides,
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UsageError::InvalidRange => write!(f, "Invalid range."),
            UsageError::StyleReadError(e) => write!(f, "Failed to read style file: {}", e),
            UsageError::InvalidFirstMoveNumber => write!(f, "Invalid first move number."),
            UsageError::InvalidLastMoveNumber => write!(f, "Invalid last move number."),
            UsageError::InvalidBoardSides => write!(f, "Invalid board sides."),
        }
    }
}

impl std::error::Error for UsageError {}
unsafe impl Send for UsageError {}
unsafe impl Sync for UsageError {}
