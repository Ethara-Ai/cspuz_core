/// Custom Minesweeper2 variant with additional constraints:
/// - Row mine cap: at most ceil(2 * cols / 3) mines per row
/// - No 2x2 mine block: no four mines forming a 2x2 square
use crate::util;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, HexInt, Optionalize, Spaces,
};
use cspuz_rs::solver::Solver;

pub fn solve_minesweeper2(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_mine = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_mine);

    // Standard: clue cells are not mines, numbered clues count 8-neighbors
    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                solver.add_expr(!is_mine.at((y, x)));
                if n < 0 {
                    continue;
                }
                solver.add_expr(is_mine.eight_neighbors((y, x)).count_true().eq(n));
            }
        }
    }

    // CUSTOM: No 2x2 mine block
    solver.add_expr(!is_mine.conv2d_and((2, 2)));

    // CUSTOM: Row mine cap — at most ceil(2 * w / 3) mines per row
    let cap = ((2 * w + 2) / 3) as i32; // ceiling division
    for y in 0..h {
        solver.add_expr(is_mine.slice_fixed_y((y, ..)).count_true().le(cap));
    }

    solver.irrefutable_facts().map(|f| f.get(is_mine))
}

type Problem = Vec<Vec<Option<i32>>>;

fn combinator() -> impl Combinator<Problem> {
    Grid::new(Choice::new(vec![
        Box::new(Optionalize::new(HexInt)),
        Box::new(Spaces::new(None, 'g')),
        Box::new(Dict::new(Some(-1), ".")),
    ]))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    problem_to_url(combinator(), "mines2", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["mines", "mines2"], url)
}
