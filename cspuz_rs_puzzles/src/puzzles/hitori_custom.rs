/// Custom Hitori variant with additional constraints:
/// - King adjacency: no two shaded cells can be diagonally adjacent
/// - Checkerboard parity: shaded cells only on (row+col) % 2 == 0
/// - Line limit: at most 2 shaded cells per row/column
use crate::util;
use cspuz_rs::graph;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, MultiDigit,
};
use cspuz_rs::solver::Solver;

pub fn solve_hitori_custom(clues: &[Vec<i32>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_black = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_black);

    // Standard: unshaded cells connected
    graph::active_vertices_connected_2d(&mut solver, !is_black);

    // Standard: no orthogonal adjacent shaded
    solver.add_expr(!is_black.conv2d_and((1, 2)));
    solver.add_expr(!is_black.conv2d_and((2, 1)));

    // CUSTOM: King adjacency — no diagonal adjacent shaded
    solver.add_expr(!is_black.conv2d_and((2, 2)));

    // CUSTOM: Checkerboard parity — shaded only on (row+col) % 2 == 0
    for y in 0..h {
        for x in 0..w {
            if (y + x) % 2 != 0 {
                solver.add_expr(!is_black.at((y, x)));
            }
        }
    }

    // CUSTOM: At most 2 shaded per row
    for y in 0..h {
        solver.add_expr(is_black.slice_fixed_y((y, ..)).count_true().le(2));
    }
    // CUSTOM: At most 2 shaded per column
    for x in 0..w {
        solver.add_expr(is_black.slice_fixed_x((.., x)).count_true().le(2));
    }

    // Standard: duplicate number enforcement
    for y in 0..h {
        for x0 in 0..w {
            for x1 in 0..x0 {
                if clues[y][x0] == clues[y][x1] && clues[y][x0] > 0 && clues[y][x1] > 0 {
                    solver.add_expr(is_black.at((y, x0)) | is_black.at((y, x1)));
                }
            }
        }
    }

    for x in 0..w {
        for y0 in 0..h {
            for y1 in 0..y0 {
                if clues[y0][x] == clues[y1][x] && clues[y0][x] > 0 && clues[y1][x] > 0 {
                    solver.add_expr(is_black.at((y0, x)) | is_black.at((y1, x)));
                }
            }
        }
    }

    solver.irrefutable_facts().map(|f| f.get(is_black))
}

type Problem = Vec<Vec<i32>>;

fn combinator() -> impl Combinator<Problem> {
    Grid::new(Choice::new(vec![
        Box::new(Dict::new(0, ".")),
        Box::new(MultiDigit::new(36, 1)),
    ]))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    problem_to_url(combinator(), "hitori_custom", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["hitori", "hitori_custom"], url)
}
