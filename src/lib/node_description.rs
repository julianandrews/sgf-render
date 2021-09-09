#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeDescription {
    pub steps: Vec<NodePathStep>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodePathStep {
    Advance(usize),
    Variation(usize),
    Last,
}

impl std::str::FromStr for NodeDescription {
    type Err = NodeDescriptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let steps =
            s.split(',')
                .map(|step| match step {
                    "last" => Ok(NodePathStep::Last),
                    _ => {
                        match step.chars().next() {
                            Some('v') => Ok(NodePathStep::Variation(step[1..].parse().map_err(
                                |_| NodeDescriptionError::InvalidVariation(step.to_owned()),
                            )?)),
                            Some(_) => Ok(NodePathStep::Advance(step.parse().map_err(|_| {
                                NodeDescriptionError::InvalidAdvance(step.to_owned())
                            })?)),
                            None => Err(NodeDescriptionError::EmptyNodePathStep),
                        }
                    }
                })
                .collect::<Result<_, _>>()?;
        Ok(NodeDescription { steps })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeDescriptionError {
    EmptyNodePathStep,
    InvalidVariation(String),
    InvalidAdvance(String),
}

impl std::fmt::Display for NodeDescriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeDescriptionError::EmptyNodePathStep => write!(f, "Empty node path step"),
            NodeDescriptionError::InvalidVariation(s) => write!(f, "Invalid variation {}", s),
            NodeDescriptionError::InvalidAdvance(s) => write!(f, "Invalid advance {}", s),
        }
    }
}

impl std::error::Error for NodeDescriptionError {}
