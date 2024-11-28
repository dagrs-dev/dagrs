use std::{
    collections::HashMap,
    panic::{self, AssertUnwindSafe},
    sync::{atomic::AtomicBool, Arc},
};

use crate::{
    connection::{in_channel::InChannel, information_packet::Content, out_channel::OutChannel},
    node::node::{Node, NodeId, NodeTable},
    utils::{env::EnvVar, execstate::ExecState},
};

use log::{debug, error};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

/// [`Graph`] is dagrs's main body.
///
/// ['Graph'] is a network that satisfies FBP logic, provides node dependencies, and runs all of its nodes completely asynchronously
/// A `Graph` contains multiple nodes, which can be added as long as they implement the [`Node`] trait.
/// Each node defines specific execution logic by implementing the [`Action`] trait and overriding the `run` method.
///
/// The execution process of a `Graph` proceeds as follows:
/// - The user creates a set of nodes, each implementing the [`Node`] trait. These nodes can be created programmatically
///   or Generate auto_node using parse.
/// - Dependencies between nodes are defined, creating a directed acyclic graph (DAG) structure.
/// - During execution, nodes communicate via input/output channels (`InChannel` and `OutChannel`).
///   These channels support both point-to-point communication (using `MPSC`) and broadcasting (using `Broadcast`).
/// - After all nodes complete their execution, marking the graph as inactive.
///   This ensures that the `Graph` cannot be executed again without resetting its state.
///
/// The [`Graph`] is designed to efficiently manage task execution with built-in fault tolerance and flexible scheduling.

pub struct Graph {
    /// Define the Net struct that holds all nodes
    nodes: HashMap<NodeId, Box<dyn Node>>,
    /// Store a task's running result.Execution results will be read
    /// and written asynchronously by several threads.
    execute_states: HashMap<NodeId, Arc<ExecState>>,
    /// Count all the nodes
    node_count: usize,
    /// Global environment variables for this Net job.
    /// It should be set before the Net job runs.
    env: Arc<EnvVar>,
    /// Mark whether the net task can continue to execute.
    /// When an error occurs during the execution of any task, This flag will still be set to true
    is_active: Arc<AtomicBool>,
}

impl Graph {
    /// Constructs a new `Graph`
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            node_count: 0,
            execute_states: HashMap::new(),
            env: Arc::new(EnvVar::new(NodeTable::default())),
            is_active: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Reset the graph state but keep the nodes.
    pub fn reset(&mut self) {
        self.execute_states = HashMap::new();
        self.env = Arc::new(EnvVar::new(NodeTable::default()));
        self.is_active = Arc::new(AtomicBool::new(true));
    }

    /// Adds a new node to the `Graph`
    pub fn add_node(&mut self, node: Box<dyn Node>) {
        self.node_count = self.node_count + 1;
        self.nodes.insert(node.id(), node);
    }
    /// Adds an edge between two nodes in the `Graph`.
    /// If the outgoing port of the sending node is empty and the number of receiving nodes is > 1, use the broadcast channel
    /// An MPSC channel is used if the outgoing port of the sending node is empty and the number of receiving nodes is equal to 1
    /// If the outgoing port of the sending node is not empty, adding any number of receiving nodes will change all relevant channels to broadcast
    pub fn add_edge(&mut self, from_id: NodeId, to_ids: Vec<NodeId>) {
        let from_node = self.nodes.get_mut(&from_id).unwrap();
        let from_channel = from_node.output_channels();
        if from_channel.0.is_empty() {
            if to_ids.len() > 1 {
                let (bcst_sender, _) = broadcast::channel::<Content>(32);
                {
                    for to_id in &to_ids {
                        from_channel
                            .insert(*to_id, Arc::new(OutChannel::Bcst(bcst_sender.clone())));
                    }
                }
                for to_id in &to_ids {
                    if let Some(to_node) = self.nodes.get_mut(to_id) {
                        let to_channel = to_node.input_channels();
                        let receiver = bcst_sender.subscribe();
                        to_channel.insert(from_id, Arc::new(Mutex::new(InChannel::Bcst(receiver))));
                    }
                }
            } else if let Some(to_id) = to_ids.get(0) {
                let (tx, rx) = mpsc::channel::<Content>(32);
                {
                    from_channel.insert(*to_id, Arc::new(OutChannel::Mpsc(tx.clone())));
                }
                if let Some(to_node) = self.nodes.get_mut(to_id) {
                    let to_channel = to_node.input_channels();
                    to_channel.insert(from_id, Arc::new(Mutex::new(InChannel::Mpsc(rx))));
                }
            }
        } else {
            let (bcst_sender, _) = broadcast::channel::<Content>(32);
            {
                for _channel in from_channel.0.values_mut() {
                    *_channel = Arc::new(OutChannel::Bcst(bcst_sender.clone()));
                }
                for to_id in &to_ids {
                    from_channel.insert(*to_id, Arc::new(OutChannel::Bcst(bcst_sender.clone())));
                }
            }
            for to_id in &to_ids {
                if let Some(to_node) = self.nodes.get_mut(to_id) {
                    let to_channel = to_node.input_channels();
                    let receiver = bcst_sender.subscribe();
                    to_channel.insert(from_id, Arc::new(Mutex::new(InChannel::Bcst(receiver))));
                }
            }
        }
    }

    /// Initializes the network, setting up the nodes.
    pub fn init(&mut self) {
        self.execute_states.reserve(self.nodes.len());
        self.nodes.values().for_each(|node| {
            self.execute_states
                .insert(node.id(), Arc::new(ExecState::new()));
        });
    }
    /// This function is used for the execution of a single net.
    pub fn run(&mut self) {
        self.init();
        if !self.is_active.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("Graph is not active. Aborting execution.");
            return;
        } else {
            for (node_id, node) in &mut self.nodes {
                let execute_state = self.execute_states[&node_id].clone();
                panic::catch_unwind(AssertUnwindSafe(|| node.run(Arc::clone(&self.env))))
                    .map_or_else(
                        |_| {
                            error!(
                                "Execution failed [name: {}, id: {}]",
                                node.name(),
                                node_id.0,
                            );
                        },
                        |out| {
                            // Store execution results
                            if out.is_err() {
                                let error = out.get_err().unwrap_or("".to_string());
                                error!(
                                    "Execution failed [name: {}, id: {}] - {}",
                                    node.name(),
                                    node_id.0,
                                    error
                                );
                                execute_state.set_output(out);
                                execute_state.exe_fail();
                            } else {
                                execute_state.set_output(out);
                                execute_state.exe_success();
                                debug!(
                                    "Execution succeed [name: {}, id: {}]",
                                    node.name(),
                                    node_id.0
                                );
                            }
                        },
                    )
            }
        }
        self.is_active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::default_node::DefaultNode;
    use crate::{
        Action, Content, EnvVar, InChannels, Node, NodeName, NodeTable, OutChannels, Output,
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    /// An implementation of [`Action`] that returns [`Output::Out`] containing a String "Hello world" from default_node.rs.
    #[derive(Default)]
    pub struct HelloAction;
    #[async_trait]
    impl Action for HelloAction {
        async fn run(&self, _: &mut InChannels, _: &OutChannels, _: Arc<EnvVar>) -> Output {
            Output::Out(Some(Content::new("Hello world".to_string())))
        }
    }

    impl HelloAction {
        pub fn new() -> Box<Self> {
            Box::new(Self::default())
        }
    }

    /// Test for execute a graph.
    ///
    /// Step 1: create a graph and two DefaultNode.
    ///
    /// Step 2: add the nodes to graph.
    ///
    /// Step 3: add the edge between Node X and "Node Y.
    ///
    /// Step 4: Run the graph and verify the output saved in the graph structure.

    #[test]
    fn test_graph_execution() {
        env_logger::init();
        let mut graph = Graph::new();
        let mut node_table = NodeTable::new();

        let node_name = "Node X";
        let node = DefaultNode::new(NodeName::from(node_name), &mut node_table);
        let node_id = node.id();

        let node1_name = "Node Y";
        let node1 = DefaultNode::with_action(
            NodeName::from(node1_name),
            HelloAction::new(),
            &mut node_table,
        );
        let node1_id = node1.id();

        graph.add_node(Box::new(node));
        graph.add_node(Box::new(node1));

        graph.add_edge(node_id, vec![node1_id]);

        graph.run();
        let out = graph.execute_states[&node1_id].get_output().unwrap();
        let out: &String = out.get().unwrap();
        assert_eq!(out, "Hello world");
    }
}
