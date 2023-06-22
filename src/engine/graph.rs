//! Task Graph
//!
//! # Graph stores dependency relations.
//!
//! [`Graph`] represents a series of tasks with dependencies, and stored in an adjacency
//! list. It must be a directed acyclic graph, that is, the dependencies of the task
//! cannot form a loop, otherwise the engine will not be able to execute the task successfully.
//! It has some useful methods for building graphs, such as: adding edges, nodes, etc.
//! And the most important of which is the `topo_sort` function, which uses topological
//! sorting to generate the execution sequence of tasks.
//!
//! # An example of a directed acyclic graph
//!
//! task1 -→ task3 ---→ task6 ----
//!  |   ↗   ↓          ↓         ↘
//!  |  /   task5 ---→ task7 ---→ task9
//!  ↓ /      ↑          ↓         ↗
//! task2 -→ task4 ---→ task8 ----
//!
//! The task execution sequence can be as follows:
//! task1->task2->task3->task4->task5->task6->task7->task8->task9
//!

use bimap::BiMap;

#[derive(Debug)]
/// Graph Struct
pub(crate) struct Graph {
    size: usize,
    /// Record node id and it's index <id,index>
    nodes: BiMap<usize, usize>,
    /// Adjacency list
    adj: Vec<Vec<usize>>,
    /// Node's in_degree, used for topological sort
    in_degree: Vec<usize>,
}

impl Graph {
    /// Allocate an empty graph
    pub(crate) fn new() -> Graph {
        Graph {
            size: 0,
            nodes: BiMap::new(),
            adj: Vec::new(),
            in_degree: Vec::new(),
        }
    }

    /// Set graph size, size is the number of tasks
    pub(crate) fn set_graph_size(&mut self, size: usize) {
        self.size = size;
        self.adj.resize(size, Vec::new());
        self.in_degree.resize(size, 0)
    }

    /// Add a node into the graph
    /// This operation will create a mapping between ID and its index.
    /// **Note:** `id` won't get repeated in dagrs,
    /// since yaml parser will overwrite its info if a task's ID repeats.
    pub(crate) fn add_node(&mut self, id: usize) {
        let index = self.nodes.len();
        self.nodes.insert(id, index);
    }

    /// Add an edge into the graph.
    /// Above operation adds a arrow from node 0 to node 1,
    /// which means task 0 shall be executed before task 1.
    pub(crate) fn add_edge(&mut self, v: usize, w: usize) {
        self.adj[v].push(w);
        self.in_degree[w] += 1;
    }

    /// Find a task's index by its ID
    pub(crate) fn find_index_by_id(&self, id: &usize) -> Option<usize> {
        self.nodes.get_by_left(id).map(|i| i.to_owned())
    }

    /// Find a task's ID by its index
    pub(crate) fn find_id_by_index(&self, index: usize) -> Option<usize> {
        self.nodes.get_by_right(&index).map(|n| n.to_owned())
    }

    /// Do topo sort in graph, returns a possible execution sequence if DAG.
    /// This operation will judge whether graph is a DAG or not,
    /// returns Some(Possible Sequence) if yes, and None if no.
    ///
    ///
    /// **Note**: this function can only be called after graph's initialization (add nodes and edges, etc.) is done.
    ///
    /// # Principle
    /// Reference: [Topological Sorting](https://www.jianshu.com/p/b59db381561a)
    ///
    /// 1. For a graph g, we record the in-degree of every node.
    ///
    /// 2. Each time we start from a node with zero in-degree, name it N0, and N0 can be executed since it has no dependency.
    ///
    /// 3. And then we decrease the in-degree of N0's children (those tasks depend on N0), this would create some new zero in-degree nodes.
    ///
    /// 4. Just repeat step 2, 3 until no more zero degree nodes can be generated.
    ///    If all tasks have been executed, then it's a DAG, or there must be a loop in the graph.
    pub(crate) fn topo_sort(&self) -> Option<Vec<usize>> {
        let mut queue = Vec::new();
        let mut in_degree = self.in_degree.clone();
        let mut count = 0;
        let mut sequence = vec![];

        in_degree
            .iter()
            .enumerate()
            .map(|(index, &degree)| {
                if degree == 0 {
                    queue.push(index)
                }
            })
            .count();

        while !queue.is_empty() {
            let v = queue.pop().unwrap(); // This unwrap is ok since `queue` is not empty

            sequence.push(v);
            count += 1;

            self.adj[v]
                .iter()
                .map(|&index| {
                    in_degree[index] -= 1;
                    if in_degree[index] == 0 {
                        queue.push(index)
                    }
                })
                .count();
        }

        if count < self.size {
            None
        } else {
            Some(sequence)
        }
    }

    /// Get the out degree of a node.
    pub(crate) fn get_node_out_degree(&self, id: &usize) -> usize {
        match self.nodes.get_by_left(id) {
            Some(index) => self.adj[*index].len(),
            None => 0,
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Graph {
            size: 0,
            nodes: BiMap::new(),
            adj: Vec::new(),
            in_degree: Vec::new(),
        }
    }
}
