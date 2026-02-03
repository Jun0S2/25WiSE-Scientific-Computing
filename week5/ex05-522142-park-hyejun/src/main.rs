// cargo run store_exchange.gph 55

/*
Write a Rust program containing a struct Graph that holds a graph,
and implements (among others) the following three functions:
1. Graph::read_from_file(filename : &str) -> Self
which reads a graph from a file
2. Graph::bfs_depth(&self, start : usize) -> usize
which runs a breadth first search (BFS) starting from node start and
computes the depth of the resulting of tree
3. Graph::connected_components(&self) -> usize
Computes the number of connected components in the graph.
4. Write down the runtime complexity of your implementation.
*/

use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

/* 1. Graph::read_from_file(filename : &str) -> Self
which reads a graph from a file */
pub struct Graph {
    pub n: usize, // Number of vertices
    pub adj_list: Vec<Vec<usize>>, // Adjacency list representation
}

impl Graph {
    pub fn read_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(filename)?; // Open the file
        let reader = BufReader::new(file); // Create a buffered reader
        let mut lines = reader.lines();

        // Read first line: "V E" where V = number of vertices, E = number of edges
        let first_line = lines.next().unwrap()?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        let n: usize = parts[0].parse()?; // Parse number of vertices
        let _m: usize = parts[1].parse()?; // Parse number of edges

        // Initialize adjacency list with n+1 vectors (1-based indexing)
        let mut adj_list = vec![vec![]; n + 1];

        // Read each edge line: "u v w" where u,v are vertices and w is weight
        for line in lines {
            let l = line?;
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() < 3 {
                continue; // Skip invalid lines
            }
            let u: usize = parts[0].parse()?; // Parse source vertex
            let v: usize = parts[1].parse()?; // Parse target vertex
            // Weight is ignored for this implementation

            // Add undirected edge to adjacency list
            adj_list[u].push(v);
            adj_list[v].push(u);
        }

        Ok(Graph { n, adj_list })
    }

    /* 2. Graph::bfs_depth(&self, start : usize) -> usize
    which runs a breadth first search (BFS) starting from node start and
    computes the depth of the resulting tree */
    pub fn bfs_depth(&self, start: usize) -> usize {
        // Check if start node is valid
        if start < 1 || start > self.n {
            return 0;
        }
        
        let mut visited = vec![false; self.n + 1]; // Track visited nodes
        let mut queue = VecDeque::new(); // BFS queue storing (node, depth)
        queue.push_back((start, 0)); // Start from given node with depth 0
        visited[start] = true; // Mark start node as visited

        let mut max_depth = 0; // Track maximum depth encountered

        // Process nodes in BFS order
        while let Some((node, depth)) = queue.pop_front() {
            max_depth = max_depth.max(depth); // Update max depth

            // Visit all unvisited neighbors
            for &neighbor in &self.adj_list[node] {
                if !visited[neighbor] {
                    visited[neighbor] = true; // Mark as visited
                    queue.push_back((neighbor, depth + 1)); // Add to queue with increased depth
                }
            }
        }

        max_depth // Return the maximum depth found
    }

    /* 3. Graph::connected_components(&self) -> usize
    Computes the number of connected components in the graph. */
    pub fn connected_components(&self) -> usize {
        let mut visited = vec![false; self.n + 1]; // Track visited nodes
        let mut count = 0; // Count of connected components

        // Check each node
        for i in 1..=self.n {
            if !visited[i] {
                count += 1; // Found a new component
                let mut queue = VecDeque::new();
                queue.push_back(i); // Start BFS from this node
                visited[i] = true; // Mark as visited

                // BFS to mark all nodes in this component
                while let Some(node) = queue.pop_front() {
                    for &neighbor in &self.adj_list[node] {
                        if !visited[neighbor] {
                            visited[neighbor] = true;
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        count // Return total number of connected components
    }
}

/* 4. Runtime complexity of implementation:
- Graph::read_from_file: O(V + E) where V is vertices, E is edges
- Graph::bfs_depth: O(V + E) - each node and edge processed once
- Graph::connected_components: O(V + E) - each node and edge processed once
Overall complexity: O(V + E)
*/

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <graph_file> <start_node>", args[0]);
        return Ok(());
    }

    let filename = &args[1]; // Graph file path
    let start_node: usize = args[2].parse()?; // Starting node for BFS

    let start_time = Instant::now(); // Start timing

    // Read graph from file
    let graph = Graph::read_from_file(filename)?;

    // Perform BFS to compute depth
    let depth = graph.bfs_depth(start_node);
    // Count connected components
    let components = graph.connected_components();

    let elapsed = start_time.elapsed(); // Calculate elapsed time

    // Output results
    println!("Depth : {}", depth);
    println!("Components: {}", components);
    println!("Time : {:.3} s", elapsed.as_secs_f64());

    Ok(())
}