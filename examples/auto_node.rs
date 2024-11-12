use std::sync::Arc;

use dagrs::{auto_node, EmptyAction, EnvVar, InChannels, Node, NodeTable, OutChannels};

#[auto_node]
struct MyNode {/*Put customized fields here.*/}

#[auto_node]
struct _MyNodeGeneric<T, 'a> {
    my_field: Vec<T>,
    my_name: &'a str,
}

#[auto_node]
struct _MyUnitNode;

fn main() {
    let mut node_table = NodeTable::default();

    let node_name = "auto_node".to_string();

    let mut s = MyNode {
        id: node_table.alloc_id_for(&node_name),
        name: node_name.clone(),
        input_channels: InChannels::default(),
        output_channels: OutChannels::default(),
        action: Box::new(EmptyAction),
    };

    assert_eq!(&s.id(), node_table.get(&node_name).unwrap());
    assert_eq!(&s.name(), &node_name);

    let output = s.run(Arc::new(EnvVar::new(NodeTable::default())));
    match output {
        dagrs::Output::Out(content) => assert!(content.is_none()),
        _ => panic!(),
    }
}
