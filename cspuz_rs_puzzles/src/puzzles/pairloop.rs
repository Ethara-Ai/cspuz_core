use crate::util;
use cspuz_rs::graph;
use cspuz_rs::serializer::{
    problem_to_url_pzprxs, url_to_problem, Choice, Combinator, Grid, NumSpaces, Spaces,
};
use cspuz_rs::solver::Solver;

/// Pairloop: single closed loop on dot grid with number clues (0-4) and arrow clues (5=↑,6=→,7=↓,8=←).
/// Arrow means 2 consecutive borders in that direction are both loop segments.
pub fn solve_pairloop(clues: &[Vec<Option<i32>>]) -> Option<graph::BoolGridEdgesIrrefutableFacts> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_line = &graph::BoolGridEdges::new(&mut solver, (h, w));
    solver.add_answer_key_bool(&is_line.horizontal);
    solver.add_answer_key_bool(&is_line.vertical);

    add_constraints(&mut solver, is_line, clues);

    solver.irrefutable_facts().map(|f| f.get(is_line))
}

pub fn enumerate_answers_pairloop(
    clues: &[Vec<Option<i32>>],
    num_max_answers: usize,
) -> Vec<graph::BoolGridEdgesModel> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_line = &graph::BoolGridEdges::new(&mut solver, (h, w));
    solver.add_answer_key_bool(&is_line.horizontal);
    solver.add_answer_key_bool(&is_line.vertical);

    add_constraints(&mut solver, is_line, clues);

    solver
        .answer_iter()
        .take(num_max_answers)
        .map(|f| f.get_unwrap(is_line))
        .collect()
}

fn add_constraints(
    solver: &mut Solver,
    is_line: &graph::BoolGridEdges,
    clues: &[Vec<Option<i32>>],
) {
    let (h, w) = util::infer_shape(clues);

    graph::single_cycle_grid_edges(solver, is_line);

    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                if n >= 0 && n <= 4 {
                    solver.add_expr(is_line.cell_neighbors((y, x)).count_true().eq(n));
                } else if n >= 5 && n <= 8 {
                    add_arrow_constraint(solver, is_line, y, x, n, h, w);
                }
            }
        }
    }
}

fn add_arrow_constraint(
    solver: &mut Solver,
    is_line: &graph::BoolGridEdges,
    y: usize,
    x: usize,
    arrow: i32,
    h: usize,
    w: usize,
) {
    match arrow {
        5 => {
            // ↑: top border of this cell + top border of cell above
            solver.add_expr(is_line.horizontal.at((y, x)));
            if y >= 1 {
                solver.add_expr(is_line.horizontal.at((y - 1, x)));
            }
        }
        6 => {
            // →: right border of this cell + right border of cell to the right
            solver.add_expr(is_line.vertical.at((y, x + 1)));
            if x + 2 <= w {
                solver.add_expr(is_line.vertical.at((y, x + 2)));
            }
        }
        7 => {
            // ↓: bottom border of this cell + bottom border of cell below
            solver.add_expr(is_line.horizontal.at((y + 1, x)));
            if y + 2 <= h {
                solver.add_expr(is_line.horizontal.at((y + 2, x)));
            }
        }
        8 => {
            // ←: left border of this cell + left border of cell to the left
            solver.add_expr(is_line.vertical.at((y, x)));
            if x >= 1 {
                solver.add_expr(is_line.vertical.at((y, x - 1)));
            }
        }
        _ => {}
    }
}

type Problem = Vec<Vec<Option<i32>>>;

pub(crate) fn combinator() -> impl Combinator<Problem> {
    Grid::new(Choice::new(vec![
        Box::new(NumSpaces::new(8, 0)),
        Box::new(Spaces::new(None, 'g')),
    ]))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    problem_to_url_pzprxs(combinator(), "pairloop", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["pairloop"], url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem_for_tests() -> Problem {
        vec![
            vec![Some(3), None, None],
            vec![Some(7), None, None],
            vec![None, None, None],
        ]
    }

    #[test]
    fn test_pairloop_solver() {
        let problem = problem_for_tests();
        let ans = solve_pairloop(&problem);
        assert!(ans.is_some());
    }

    #[test]
    fn test_pairloop_serializer() {
        let problem = problem_for_tests();
        let url = serialize_problem(&problem);
        assert!(url.is_some());
        let url = url.unwrap();
        let deserialized = deserialize_problem(&url);
        assert!(deserialized.is_some());
        assert_eq!(deserialized.unwrap(), problem);
    }
}
