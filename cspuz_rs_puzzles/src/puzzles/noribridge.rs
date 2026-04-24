use cspuz_rs::graph::{self, Graph, active_vertices_connected_via_active_edges};
use cspuz_rs::serializer::{
    problem_to_url_with_context, url_to_problem, Choice, Combinator, Context, Dict, HexInt,
    Optionalize, RoomsWithValues, Size, Spaces,
};
use cspuz_rs::solver::{count_true, Solver};

/// Nori Bridge: place bridges on region borders.
/// Rules:
///   1. Bridges on border segments between adjacent regions
///   2. All regions connected via bridges
///   3. Numbered regions must have exactly that many bridges
///   4. At most 1 bridge per shared border between two regions
///
/// Encoding: same as heyawake — rooms with optional numeric values.
/// Solver output: Vec<Option<bool>> indexed by region-adjacency edges.

pub fn solve_noribridge(
    borders: &graph::InnerGridEdges<Vec<Vec<bool>>>,
    clues: &[Option<i32>],
) -> Option<(Vec<Option<bool>>, Vec<(usize, usize)>)> {
    let rooms = graph::borders_to_rooms(borders);
    let n_rooms = rooms.len();
    assert_eq!(n_rooms, clues.len());

    let (h, w) = borders.base_shape();
    let mut cell_room = vec![vec![0usize; w]; h];
    for (room_id, cells) in rooms.iter().enumerate() {
        for &(y, x) in cells {
            cell_room[y][x] = room_id;
        }
    }

    // Collect unique adjacent region pairs → edges of abstract region graph
    let mut edge_set = std::collections::BTreeSet::new();
    for y in 0..h {
        for x in 0..w {
            if x + 1 < w && cell_room[y][x] != cell_room[y][x + 1] {
                let a = cell_room[y][x];
                let b = cell_room[y][x + 1];
                edge_set.insert((a.min(b), a.max(b)));
            }
            if y + 1 < h && cell_room[y][x] != cell_room[y + 1][x] {
                let a = cell_room[y][x];
                let b = cell_room[y + 1][x];
                edge_set.insert((a.min(b), a.max(b)));
            }
        }
    }
    let edges: Vec<(usize, usize)> = edge_set.into_iter().collect();
    let n_edges = edges.len();

    let mut region_graph = Graph::new(n_rooms);
    for &(u, v) in &edges {
        region_graph.add_edge(u, v);
    }

    // incident[room_id] = list of edge indices touching that room
    let mut incident: Vec<Vec<usize>> = vec![vec![]; n_rooms];
    for (i, &(u, v)) in edges.iter().enumerate() {
        incident[u].push(i);
        incident[v].push(i);
    }

    let mut solver = Solver::new();

    let bridge = &solver.bool_var_1d(n_edges);
    solver.add_answer_key_bool(bridge);

    let all_active = &solver.bool_var_1d(n_rooms);
    for i in 0..n_rooms {
        solver.add_expr(all_active.at(i));
    }
    active_vertices_connected_via_active_edges(&mut solver, all_active, bridge, &region_graph);

    for (room_id, clue) in clues.iter().enumerate() {
        if let Some(n) = clue {
            let inc = &incident[room_id];
            let bridge_vars: Vec<_> = inc.iter().map(|&e| bridge.at(e)).collect();
            solver.add_expr(count_true(&bridge_vars).eq(*n));
        }
    }

    let result = solver.irrefutable_facts().map(|f| f.get(bridge));
    result.map(|r| (r, edges))
}

pub(super) type Problem = (graph::InnerGridEdges<Vec<Vec<bool>>>, Vec<Option<i32>>);

pub(super) fn combinator() -> impl Combinator<Problem> {
    Size::new(RoomsWithValues::new(Choice::new(vec![
        Box::new(Optionalize::new(HexInt)),
        Box::new(Spaces::new(None, 'g')),
        Box::new(Dict::new(Some(-1), ".")),
    ])))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    let height = problem.0.vertical.len();
    let width = problem.0.vertical[0].len() + 1;
    problem_to_url_with_context(
        combinator(),
        "noribridge",
        problem.clone(),
        &Context::sized(height, width),
    )
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["noribridge"], url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noribridge_solver() {
        let url = "https://puzz.link/p?noribridge/4/2/a0h1g1";
        let problem = deserialize_problem(url);
        assert!(problem.is_some());
        let (borders, clues) = problem.unwrap();
        let result = solve_noribridge(&borders, &clues);
        assert!(result.is_some());
    }

    #[test]
    fn test_noribridge_all_urls() {
        // Easy 6×6
        let borders_e = graph::InnerGridEdges {
            horizontal: vec![
                vec![false, false, false, false, false, false],
                vec![true, true, true, true, true, true],
                vec![false, false, false, false, false, false],
                vec![false, false, false, false, false, false],
                vec![false, false, false, false, false, false],
            ],
            vertical: vec![
                vec![false, true, false, true, false],
                vec![false, true, false, true, false],
                vec![false, true, false, true, false],
                vec![false, true, false, true, false],
                vec![false, true, false, true, false],
                vec![false, true, false, true, false],
            ],
        };
        let clues_e = vec![Some(2), Some(3), Some(2), Some(1), Some(1), Some(1)];
        let url_e = serialize_problem(&(borders_e, clues_e)).unwrap();
        eprintln!("EASY: {}", url_e);

        // Medium 8×8
        let borders_m = graph::InnerGridEdges {
            horizontal: vec![
                vec![false; 8], vec![false; 8], vec![false; 8],
                vec![true; 8],
                vec![false; 8], vec![false; 8], vec![false; 8],
            ],
            vertical: vec![
                vec![false, true, false, true, false, true, false],
                vec![false, true, false, true, false, true, false],
                vec![false, true, false, true, false, true, false],
                vec![false, true, false, true, false, true, false],
                vec![false, true, false, true, false, true, false],
                vec![false, true, false, true, false, true, false],
                vec![false, true, false, true, false, true, false],
                vec![false, true, false, true, false, true, false],
            ],
        };
        let clues_m = vec![None, Some(3), Some(3), None, Some(1), Some(1), Some(1), Some(1)];
        let url_m = serialize_problem(&(borders_m, clues_m)).unwrap();
        eprintln!("MEDIUM: {}", url_m);

        // Hard 10×10
        let borders_h = graph::InnerGridEdges {
            horizontal: vec![
                vec![false; 10],
                vec![true; 10],
                vec![false; 10],
                vec![true; 10],
                vec![false; 10],
                vec![true; 10],
                vec![false; 10], vec![false; 10], vec![false; 10],
            ],
            vertical: vec![
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
                vec![false, true, false, true, false, true, false, false, false],
            ],
        };
        let clues_h = vec![Some(2), Some(3), Some(3), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(2), Some(1), Some(1), Some(1), Some(1)];
        let url_h = serialize_problem(&(borders_h, clues_h)).unwrap();
        eprintln!("HARD: {}", url_h);
    }
}
