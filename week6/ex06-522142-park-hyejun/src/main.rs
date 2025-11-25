/**
* Tested result : 
cargo run b15.gph 1 100
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/ex06-522142-park-hyejun b15.gph 1 100`
b15.gph MST=462 SP=36 Path: 1 5 25 68 34 100 Time: 1 ms
*/
use std::collections::{BinaryHeap, HashMap, HashSet};   // For graph representation and priority queue
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

#[derive(Debug)]
struct Graph {
    nodes: HashMap<u32, Vec<(u32, u32)>>, // node -> list of (neighbor, weight)
}

impl Graph {
    fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
        }
    }

    fn add_edge(&mut self, u: u32, v: u32, weight: u32) {               // Undirected graph : Connect edge u <-> v
        self.nodes.entry(u).or_insert_with(Vec::new).push((v, weight));
        self.nodes.entry(v).or_insert_with(Vec::new).push((u, weight)); 
    }

    // Prim's algorithm for MST
    fn mst_weight(&self) -> u32 {   // Return total weight of MST
        if self.nodes.is_empty() {  // Empty graph
            return 0;
        }

        let mut visited = HashSet::new();
        let mut heap = BinaryHeap::new();
        let mut total_weight = 0;

        // Start with first node
        let start_node = *self.nodes.keys().next().unwrap();
        visited.insert(start_node);

        // Add all edges from start node to heap
        for &(neighbor, weight) in &self.nodes[&start_node] {
            heap.push(std::cmp::Reverse((weight, neighbor)));
        }

        while visited.len() < self.nodes.len() {
            if let Some(std::cmp::Reverse((weight, to))) = heap.pop() { // Get edge with smallest weight
                if visited.contains(&to) {  // Already visited
                    continue;
                }

                visited.insert(to); // Mark node as visited
                total_weight += weight; // Add weight to total

                // Add edges from the new node
                for &(neighbor, next_weight) in &self.nodes[&to] {
                    if !visited.contains(&neighbor) {
                        heap.push(std::cmp::Reverse((next_weight, neighbor)));
                    }
                }
            } else {
                break; // Disconnected graph
            }
        }

        total_weight
    }

    // Dijkstra's algorithm for shortest path
    fn shortest_path(&self, start: u32, end: u32) -> Option<(u32, Vec<u32>)> {
        if !self.nodes.contains_key(&start) || !self.nodes.contains_key(&end) { // Invalid nodes
            return None;
        }

        let mut dist: HashMap<u32, u32> = HashMap::new();
        let mut prev: HashMap<u32, u32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        // Initialize distances
        dist.insert(start, 0);
        heap.push(std::cmp::Reverse((0, start)));

        for &node in self.nodes.keys() {    // Set initial distances to infinity
            if node != start {              // except start node
                dist.insert(node, u32::MAX);
            }
        }

        while let Some(std::cmp::Reverse((cost, node))) = heap.pop() {  // Get node with smallest cost
            if node == end {    // Reached destination(end case)
                break;
            }

            if cost > dist[&node] { // Already found a better path
                continue;
            }

            if let Some(neighbors) = self.nodes.get(&node) {    // Explore neighbors
                for &(neighbor, weight) in neighbors {          // For each neighbor
                    let next_cost = cost + weight;              // Calculate new cost
                    if next_cost < *dist.get(&neighbor).unwrap_or(&u32::MAX) {  // Found a better path
                        dist.insert(neighbor, next_cost);                       // Update distance
                        prev.insert(neighbor, node);                            // Update previous node
                        heap.push(std::cmp::Reverse((next_cost, neighbor)));    // Push to heap
                    }
                }
            }
        }

        if !dist.contains_key(&end) || dist[&end] == u32::MAX { // No path found
            return None;
        }

        // Reconstruct path
        let mut path = Vec::new();  // To store the path
        let mut current = end;      //  Start from end node
        while current != start  {   // Backtrack to start node
            path.push(current);      // Add current node to path
            current = prev[&current];   // Move to previous node
        }
        path.push(start);       // Add start node
        path.reverse();         // Reverse to get correct order since i pushed from end to start

        Some((dist[&end], path))    // Return total distance and path
    }
}

fn read_graph(filename: &str) -> Result<Graph, std::io::Error> {    // Read graph from file
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut graph = Graph::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            let u: u32 = parts[0].parse().unwrap();
            let v: u32 = parts[1].parse().unwrap();
            let weight: u32 = parts[2].parse().unwrap();
            graph.add_edge(u, v, weight);
        }
    }

    Ok(graph)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <graph_file> <start_vertex> <end_vertex>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];    
    let start_vertex: u32 = args[2].parse().expect("Invalid start vertex");
    let end_vertex: u32 = args[3].parse().expect("Invalid end vertex");

    let total_start = Instant::now();

    let graph = match read_graph(filename) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error reading graph file: {}", e);
            std::process::exit(1);
        }
    };

    let mst_weight = graph.mst_weight(); // Calculate MST weight

     // Calculate shortest path

    let (sp_length, sp_path) = match graph.shortest_path(start_vertex, end_vertex) {
        Some((length, path)) => (length, path), // Found path
        None => {   // No path found
            eprintln!("No path found between {} and {}", start_vertex, end_vertex);
            std::process::exit(1);
        }
    };

    let total_time = total_start.elapsed();

    println!("{} MST={} SP={} Path: {} Time: {} ms", 
        filename, 
        mst_weight, 
        sp_length,
        sp_path.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(" "),
        total_time.as_millis()
    );
}