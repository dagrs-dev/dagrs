//! Graph stores dependency relations

use bimap::BiMap;

#[derive(Debug)]
/// Graph Struct
pub struct Graph {
    size: usize,
    /// Record node id and it's index
    nodes: BiMap<usize, usize>,
    /// Adjacency list
    adj: Vec<Vec<usize>>,
    /// Node's indegree, used for topo sort
    indegree: Vec<usize>,
}

impl Graph {
    /// Allocate an empty graph
    /// 
    /// # Example
    /// ```
    /// let g = Grapg::new();
    /// ```
    pub fn new() -> Graph {
        Graph {
            size: 0,
            nodes: BiMap::new(),
            adj: Vec::new(),
            indegree: Vec::new(),
        }
    }

    /// Set graph size, size is the number of tasks
    /// 
    /// # Example
    /// ```
    /// let size = 10; // 10 nodes
    /// g.set_graph_size(size);
    /// ```
    pub fn set_graph_size(&mut self, size: usize) {
        self.size = size;
        self.adj.resize(size, Vec::new());
        self.indegree.resize(size, 0)
    }

    /// Add a node into the graph
    /// 
    /// This operation will create a mapping between ID and its index.
    /// 
    /// # Example
    /// ```
    /// g.add_node("Node1");
    /// ```
    /// **Note:** `id` won't get repeated in dagrs,
    /// since yaml parser will overwrite its info if a task's ID repeats.
    pub fn add_node(&mut self, id: usize) {
        let index = self.nodes.len();
        self.nodes.insert(id, index);
    }

    /// Add an edge into the graph.
    /// 
    /// # Example
    /// ```
    /// g.add_edge(0, 1);
    /// ```
    /// Above operation adds a arrow from node 0 to node 1,
    /// which means task 0 shall be executed before task 1.
    pub fn add_edge(&mut self, v: usize, w: usize) {
        self.adj[v].push(w);
        self.indegree[w] += 1;
    }

    /// Find a task's index by its ID
    pub fn find_index_by_id(&self, id: &usize) -> Option<usize> {
        self.nodes.get_by_left(id).map(|i| i.to_owned())
    }

    /// Find a task's ID by its index
    pub fn find_id_by_index(&self, index: usize) -> Option<usize> {
        self.nodes.get_by_right(&index).map(|n| n.to_owned())
    }

    /// Do topo sort in graph, returns a possible execution sequnce if DAG
    /// 
    /// # Example
    /// ```
    /// g.topo_sort();
    /// ```
    /// This operation will judge whether graph is a DAG or not, 
    /// returns Some(Possible Sequence) if yes, and None if no.
    /// 
    /// 
    /// **Note**: this function can only be called after graph's initialization (add nodes and edges, etc.) is done.
    /// 
    /// # Principle
    /// Reference: [Topological Sorting](https://www.jianshu.com/p/b59db381561a)
    /// 
    /// 1. For a grapg g, we record the indgree of every node.
    /// 
    /// 2. Each time we start from a node with zero indegree, name it N0, and N0 can be executed since it has no dependency.
    /// 
    /// 3. And then we decrease the indegree of N0's children (those tasks depend on N0), this would create some new zero indegree nodes.
    /// 
    /// 4. Just repeat step 2, 3 until no more zero degree nodes can be generated.
    ///    If all tasks have been executed, then it's a DAG, or there must be a loop in the graph.
    pub fn topo_sort(&self) -> Option<Vec<usize>> {
        let mut queue = Vec::new();
        let mut indegree = self.indegree.clone();
        let mut count = 0;
        let mut sequence = vec![];

        indegree
            .iter()
            .enumerate()
            .map(|(index, &degree)| {
                if degree == 0 {
                    queue.push(index)
                }
            })
            .count();

        while !queue.is_empty() {
            let v = queue.pop().unwrap();   // This unwrap is ok since `queue` is not empty

            sequence.push(v);
            count += 1;

            self.adj[v]
                .iter()
                .map(|&index| {
                    indegree[index] -= 1;
                    if indegree[index] == 0 {
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
}
