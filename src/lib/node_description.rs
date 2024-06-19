use clap::Parser;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Parser)]
pub struct NodeDescription {
    /// Game number to display (for multi-game files).
    #[arg(short, long, default_value_t = 0)]
    pub game_number: u64,
    /// Variation number to display (use `query` command for numbers).
    #[arg(short, long, default_value_t = 0)]
    pub variation: u64,
    /// Node number in the variation to display.
    #[arg(short, long, default_value = "last")]
    pub node_number: NodeNumber,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeNumber {
    Number(u64),
    Last,
}

impl std::str::FromStr for NodeNumber {
    type Err = NodeDescriptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "last" => Ok(NodeNumber::Last),
            _ => {
                let n = s
                    .parse()
                    .map_err(|_| NodeDescriptionError::UnrecognizedNodeNumber(s.to_string()))?;
                Ok(NodeNumber::Number(n))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeDescriptionError {
    UnrecognizedNodeNumber(String),
}

impl std::fmt::Display for NodeDescriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeDescriptionError::UnrecognizedNodeNumber(s) => {
                write!(f, "Unrecognized node number: {}", s)
            }
        }
    }
}

impl std::error::Error for NodeDescriptionError {}
