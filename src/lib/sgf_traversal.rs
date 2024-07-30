use sgf_parse::{go::Prop, SgfNode};

use crate::errors::GobanError;

/// Returns an iterator over SgfTraversalNode values at the root of each variation.
pub fn variation_roots(node: &SgfNode<Prop>) -> impl Iterator<Item = SgfTraversalNode<'_>> {
    SgfTraversal::new(node).filter(|node| node.is_variation_root)
}

/// Returns an iterator of SgfTraversalNode values for every node in a variation.
pub fn variation_nodes(
    root: &SgfNode<Prop>,
    variation: u64,
) -> Result<impl Iterator<Item = SgfTraversalNode<'_>>, GobanError> {
    let mut parents = vec![0; variation as usize + 1];
    let mut starts = vec![u64::MAX; variation as usize + 1];
    let mut variation_seen = false;
    for node in variation_roots(root).take_while(|n| n.variation <= variation) {
        parents[node.variation as usize] = node.parent_variation;
        starts[node.variation as usize] =
            starts[node.variation as usize].min(node.variation_node_number);
        if node.variation == variation {
            variation_seen = true;
        }
    }
    if !variation_seen {
        return Err(GobanError::MissingVariation);
    }
    let mut current_variation = variation;
    let mut variations = vec![];
    while current_variation > 0 {
        let parent = parents[current_variation as usize];
        let start = starts[current_variation as usize];
        variations.push((current_variation, start));
        current_variation = parent;
    }

    let mut current_variation = 0;
    let (mut next_variation, mut next_node_number) = variations.pop().unwrap_or((0, 0));
    Ok(SgfTraversal::new(root)
        .take_while(move |node| node.variation <= variation)
        .filter(move |node| {
            if node.variation == current_variation && node.variation_node_number >= next_node_number
            {
                current_variation = next_variation;
                if let Some((a, b)) = variations.pop() {
                    (next_variation, next_node_number) = (a, b);
                }
            }
            node.variation == current_variation
        }))
}

#[derive(Debug, Clone)]
pub struct SgfTraversal<'a> {
    stack: Vec<SgfTraversalNode<'a>>,
    variation: u64,
}

impl<'a> SgfTraversal<'a> {
    pub fn new(sgf_node: &'a SgfNode<Prop>) -> Self {
        SgfTraversal {
            stack: vec![SgfTraversalNode {
                sgf_node,
                variation_node_number: 0,
                variation: 0,
                parent_variation: 0,
                branch_number: 0,
                is_variation_root: true,
                branches: vec![],
            }],
            variation: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SgfTraversalNode<'a> {
    pub sgf_node: &'a SgfNode<Prop>,
    pub variation_node_number: u64,
    pub variation: u64,
    pub parent_variation: u64,
    pub branch_number: u64,
    pub is_variation_root: bool,
    pub branches: Vec<bool>,
}

impl<'a> Iterator for SgfTraversal<'a> {
    type Item = SgfTraversalNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut traversal_node = self.stack.pop()?;
        let sgf_node = traversal_node.sgf_node;
        let variation_node_number = traversal_node.variation_node_number + 1;
        let is_variation_root = sgf_node.children.len() > 1;
        if traversal_node.is_variation_root && traversal_node.branch_number != 0 {
            self.variation += 1;
            traversal_node.parent_variation = traversal_node.variation;
            traversal_node.variation = self.variation;
        }
        for (branch_number, child) in sgf_node.children.iter().enumerate().rev() {
            let mut branches = traversal_node.branches.clone();
            if is_variation_root {
                if branch_number == sgf_node.children.len() - 1 {
                    branches.push(false);
                } else {
                    branches.push(true);
                }
            }
            self.stack.push(SgfTraversalNode {
                sgf_node: child,
                variation_node_number,
                variation: traversal_node.variation,
                parent_variation: traversal_node.parent_variation,
                branch_number: branch_number as u64,
                is_variation_root,
                branches,
            });
        }
        Some(traversal_node)
    }
}
