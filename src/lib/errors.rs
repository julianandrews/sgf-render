use sgf_parse::SgfParseError;

#[derive(Debug)]
pub enum GobanError {
    ParseError(SgfParseError),
    StyleDefError(minidom::Error),
    InsufficientSgfNodes,
    MissingGame,
    MissingVariation,
    InvalidMove,
    InvalidRange,
    UnlabellableRange,
}

impl std::fmt::Display for GobanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(e) => write!(f, "{}", e),
            Self::StyleDefError(e) => write!(f, "Invalid defs in style: {}", e),
            Self::InvalidMove => write!(f, "Invalid move"),
            Self::InsufficientSgfNodes => write!(f, "Insufficient SGF nodes found"),
            Self::MissingGame => write!(f, "Selected game not found"),
            Self::MissingVariation => write!(f, "Selected variation not found"),
            Self::InvalidRange => write!(f, "Invalid range to render in goban"),
            Self::UnlabellableRange => write!(f, "Range too large for use with labels"),
        }
    }
}

impl std::error::Error for GobanError {}

impl From<SgfParseError> for GobanError {
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
    InvalidNodeNumber(String),
    InvalidTextOutputOption(String),
    InvalidTileSet,
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UsageError::InvalidRange => write!(f, "Invalid range"),
            UsageError::StyleReadError(e) => write!(f, "Failed to read style file: {}", e),
            UsageError::InvalidFirstMoveNumber => write!(f, "Invalid first move number"),
            UsageError::InvalidLastMoveNumber => write!(f, "Invalid last move number"),
            UsageError::InvalidBoardSides => write!(f, "Invalid board sides"),
            UsageError::InvalidNodeNumber(s) => write!(f, "Invalid node number '{}'", s),
            UsageError::InvalidTextOutputOption(s) => {
                write!(f, "{} not supported for text output", s)
            }
            UsageError::InvalidTileSet => write!(f, "Must be 11 characters long"),
        }
    }
}

impl std::error::Error for UsageError {}
unsafe impl Send for UsageError {}
unsafe impl Sync for UsageError {}

#[derive(Debug)]
pub enum QueryError {
    ParseError(SgfParseError),
    IoError(std::io::Error),
    GameNotFound,
    VariationNotFound,
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QueryError::ParseError(e) => write!(f, "{}", e),
            QueryError::IoError(e) => write!(f, "{}", e),
            QueryError::GameNotFound => write!(f, "Game not found."),
            QueryError::VariationNotFound => write!(f, "Variation not found."),
        }
    }
}

impl std::error::Error for QueryError {}

impl From<SgfParseError> for QueryError {
    fn from(error: SgfParseError) -> Self {
        Self::ParseError(error)
    }
}

impl From<std::io::Error> for QueryError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}
