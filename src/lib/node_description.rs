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

impl NodeDescription {
    pub fn default(kifu_mode: bool) -> NodeDescription {
        match kifu_mode {
            true => NodeDescription {
                steps: vec![NodePathStep::Last],
            },
            false => NodeDescription {
                steps: vec![NodePathStep::Advance(0)],
            },
        }
    }

    pub fn normalize(&mut self) {
        for step in &mut self.steps {
            if matches!(step, NodePathStep::Variation(0)) {
                *step = NodePathStep::Advance(1);
            }
        }
        let mut steps = vec![];
        for step in &self.steps {
            match step {
                NodePathStep::Variation(_) | NodePathStep::Last => steps.push(*step),
                NodePathStep::Advance(n) => {
                    if let Some(NodePathStep::Advance(m)) = steps.last() {
                        *steps.last_mut().unwrap() = NodePathStep::Advance(n + m)
                    } else {
                        steps.push(NodePathStep::Advance(*n));
                    }
                }
            }
        }
        self.steps = steps;
    }
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

impl std::fmt::Display for NodePathStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodePathStep::Advance(n) => write!(f, "{}", n),
            NodePathStep::Variation(n) => write!(f, "v{}", n),
            NodePathStep::Last => write!(f, "last"),
        }
    }
}

impl std::fmt::Display for NodeDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut steps = vec![];
        for step in &self.steps {
            steps.push(step.to_string());
        }
        write!(f, "{}", steps.join(","))
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
