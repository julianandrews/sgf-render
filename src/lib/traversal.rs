use sgf_parse::{go::Prop, SgfNode};

pub fn variation_roots(node: &SgfNode<Prop>) -> impl Iterator<Item = SgfTraversalNode<'_>> {
    SgfTraversal::new(node).filter(|node| node.is_variation_root)
}

#[derive(Debug, Clone)]
pub struct SgfTraversal<'a> {
    stack: Vec<SgfTraversalNode<'a>>,
    variation: usize,
}

impl<'a> SgfTraversal<'a> {
    pub fn new(sgf_node: &'a SgfNode<Prop>) -> Self {
        SgfTraversal {
            stack: vec![SgfTraversalNode {
                sgf_node,
                move_number: 0,
                variation: 0,
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
    pub move_number: usize,
    pub variation: usize,
    pub branch_number: usize,
    pub is_variation_root: bool,
    pub branches: Vec<bool>,
}

impl<'a> Iterator for SgfTraversal<'a> {
    type Item = SgfTraversalNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut traversal_node = self.stack.pop()?;
        let sgf_node = traversal_node.sgf_node;
        let move_number = traversal_node.move_number + 1;
        let is_variation_root = sgf_node.children.len() > 1;
        if traversal_node.is_variation_root && traversal_node.branch_number != 0 {
            self.variation += 1;
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
                move_number,
                variation: traversal_node.variation,
                branch_number,
                is_variation_root,
                branches,
            });
        }
        Some(traversal_node)
    }
}
