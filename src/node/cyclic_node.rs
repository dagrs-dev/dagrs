use std::sync::Arc;

use async_trait::async_trait;

use crate::{EnvVar, InChannels, Node, NodeId, NodeName, OutChannels, Output};

/// # Cycle
/// A `Cycle` represents a sub graph, which contains a group of
/// [`Node`]s connected in a circle.
pub struct Cycle {
    entry: NodeId,
    exit: NodeId,
    times: usize,
    nodes: Vec<Box<dyn Node>>,
    ids: Vec<NodeId>,
    inner_edges: Vec<(NodeId, Vec<NodeId>)>,
}

impl Cycle {
    pub fn new(
        nodes: Vec<impl Node + 'static>,
        entry: NodeId,
        exit: NodeId,
        times: usize,
    ) -> Result<Self, CyclicNodeErr> {
        let mut cycle = Self {
            entry,
            exit,
            times,
            nodes: vec![],
            ids: vec![],
            inner_edges: vec![],
        };

        // let (mut valid_entry, mut valid_exit)
        if nodes
            .iter()
            .filter(|node| node.id() == entry)
            .next()
            .is_none()
        {
            Err(CyclicNodeErr::InvalidEntry(entry))
        } else if nodes
            .iter()
            .filter(|node| node.id() == exit)
            .next()
            .is_none()
        {
            Err(CyclicNodeErr::InvalidExit(exit))
        } else {
            nodes
                .into_iter()
                .for_each(|node| cycle.nodes.push(Box::new(node)));
            Ok(cycle)
        }
    }

    // pub fn add_inner_edge(&mut self, src: NodeId, dests: Vec<NodeId>) -> Result<(), CyclicNodeErr> {
    //     self.inner_edges.push((src, dests));
    // }
}

#[async_trait]
impl Node for Cycle {
    fn id(&self) -> NodeId {
        panic!()
    }

    fn name(&self) -> NodeName {
        panic!()
    }

    fn input_channels(&mut self) -> &mut InChannels {
        panic!()
    }

    fn output_channels(&mut self) -> &mut OutChannels {
        panic!()
    }

    async fn run(&mut self, _: Arc<EnvVar>) -> Output {
        panic!("CyclicNode itself should never run.")
    }
}

pub enum CyclicNodeErr {
    InvalidEntry(NodeId),
    InvalidExit(NodeId),
    InvalidInnerEdge((NodeId, NodeId)),
}

pub(crate) struct CyclicMark {
    entry: NodeId,
    exit: NodeId,
    times: usize,
    nodes: Vec<NodeId>,
    inner_edges: Vec<(NodeId, Vec<NodeId>)>,
}
