use std::sync::Arc;

use dagrs::{
    auto_node, dependencies,
    graph::{self, graph::Graph},
    EmptyAction, EnvVar, InChannels, Node, NodeTable, OutChannels,
};

#[auto_node]
struct MyNode {/*Put customized fields here.*/}

fn main() {
    let mut node_table = NodeTable::default();

    let node_name = "auto_node".to_string();

    let s = MyNode {
        id: node_table.alloc_id_for(&node_name),
        name: node_name.clone(),
        input_channels: InChannels::default(),
        output_channels: OutChannels::default(),
        action: Box::new(EmptyAction),
    };

    let a = MyNode {
        id: node_table.alloc_id_for(&node_name),
        name: node_name.clone(),
        input_channels: InChannels::default(),
        output_channels: OutChannels::default(),
        action: Box::new(EmptyAction),
    };

    let b = MyNode {
        id: node_table.alloc_id_for(&node_name),
        name: node_name.clone(),
        input_channels: InChannels::default(),
        output_channels: OutChannels::default(),
        action: Box::new(EmptyAction),
    };
    let mut g = dependencies!(s -> a b,
     b -> a
    );

    g.run();
}
