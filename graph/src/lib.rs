mod graph {
    use std::collections::vec_deque::VecDeque;

    #[derive(Debug)]
    pub struct DirectedGraph {
        adjacency_list: Vec<Vec<usize>>
    }

    impl DirectedGraph {
        pub fn new(vertices: usize) -> DirectedGraph {
            DirectedGraph {
                adjacency_list: vec![vec![]; vertices],
            }
        }

        pub fn add_edges(&mut self, from: usize, outgoing_edges: &[usize]) {
            self.adjacency_list[from].extend_from_slice(outgoing_edges);
        }

        pub fn topological_sort(&self) -> Option<Vec<usize>> {
            let mut in_degrees: Vec<usize> = vec![0; self.adjacency_list.len()];
            for adj_list in &self.adjacency_list {
                for vertex in adj_list {
                    in_degrees[*vertex] += 1;
                }
            }

            let mut vertices_with_no_incoming: VecDeque<usize> = VecDeque::with_capacity(self.adjacency_list.len());
            for (vertex, in_degree) in in_degrees.iter().enumerate() {
                if *in_degree == 0 {
                    vertices_with_no_incoming.push_back(vertex);
                }
            }

            let mut result: Vec<usize> = Vec::with_capacity(self.adjacency_list.len());
            while !vertices_with_no_incoming.is_empty() {
                let vertex = vertices_with_no_incoming.pop_front().unwrap();
                result.push(vertex);

                for neighbor in &self.adjacency_list[vertex] {
                    in_degrees[*neighbor] -= 1;

                    if in_degrees[*neighbor] == 0 {
                        vertices_with_no_incoming.push_back(*neighbor);
                    }
                };
            }

            if result.len() != in_degrees.len() {
                None
            } else {
                Some(result)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::graph::DirectedGraph;

        #[test]
        fn create_new_graph() {
            let graph = DirectedGraph::new(10);
            assert_eq!(10, graph.adjacency_list.capacity());
        }

        #[test]
        fn add_edges() {
            let mut graph = DirectedGraph::new(10);
            graph.add_edges(0, &[1, 2, 3]);
            graph.add_edges(1, &[2, 3]);
            graph.add_edges(2, &[3]);

            assert_eq!(vec![1, 2, 3], graph.adjacency_list[0]);
            assert_eq!(vec![2, 3], graph.adjacency_list[1]);
            assert_eq!(vec![3], graph.adjacency_list[2]);
        }

        #[test]
        fn test_topo_sort_simple_graph() {
            let mut graph = DirectedGraph::new(5);
            graph.add_edges(0, &[1, 2, 3]);
            graph.add_edges(1, &[2, 3]);
            graph.add_edges(2, &[3]);

            let sorted_vertices = graph.topological_sort();
            assert_eq!(Some(vec![0, 4, 1, 2, 3]), sorted_vertices);
        }

        #[test]
        fn test_topo_sort_circle() {
            let mut graph = DirectedGraph::new(2);
            graph.add_edges(0, &[1]);
            graph.add_edges(1, &[0]);

            let sorted_vertices = graph.topological_sort();
            assert_eq!(None, sorted_vertices);
        }
    }
}
