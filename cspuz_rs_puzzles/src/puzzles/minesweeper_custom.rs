/// Custom Minesweeper with no 2x2 mine block + density <= 25%
use crate::util;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, HexInt, Optionalize, Spaces,
};
use cspuz_rs::solver::Solver;

pub fn solve_minesweeper_custom(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_mine = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_mine);

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

    // CUSTOM: Mine density <= 25%
    let total = (h * w) as i32;
    let max_mines = total / 4;
    solver.add_expr(is_mine.count_true().le(max_mines));

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
    problem_to_url(combinator(), "mines", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["mines", "minesweeper_custom"], url)
}
