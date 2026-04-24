/// Custom Sudoku2 variant with additional constraint:
/// - Even-digit balance: each row must have exactly 4 even digits (2, 4, 6, 8)
/// Only applies to 9x9 sudoku.
use crate::util;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, HexInt, Optionalize, Spaces,
};
use cspuz_rs::solver::{count_true, Solver};

pub fn solve_sudoku2(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<i32>>>> {
    let (h, w) = util::infer_shape(clues);
    if h != w {
        return None;
    }
    let n = h;
    let (bh, bw) = match n {
        4 => (2, 2),
        6 => (2, 3),
        9 => (3, 3),
        16 => (4, 4),
        25 => (5, 5),
        _ => return None,
    };

    let mut solver = Solver::new();
    let num = &solver.int_var_2d((n, n), 1, n as i32);
    solver.add_answer_key_int(num);

    // Standard: all_different rows, cols, boxes
    for i in 0..n {
        solver.all_different(num.slice_fixed_y((i, ..)));
        solver.all_different(num.slice_fixed_x((.., i)));
    }
    for i in 0..bw {
        for j in 0..bh {
            solver
                .all_different(num.slice((((i * bh)..((i + 1) * bh)), ((j * bw)..((j + 1) * bw)))));
        }
    }

    // Standard: clue placement
    for y in 0..n {
        for x in 0..n {
            if let Some(val) = clues[y][x] {
                if val > 0 {
                    solver.add_expr(num.at((y, x)).eq(val));
                }
            }
        }
    }

    // CUSTOM: Even-digit balance — each row has exactly 4 even digits (for 9x9)
    if n == 9 {
        for y in 0..n {
            let mut is_even = vec![];
            for x in 0..n {
                // Even digits: 2, 4, 6, 8
                is_even.push(
                    num.at((y, x)).eq(2)
                        | num.at((y, x)).eq(4)
                        | num.at((y, x)).eq(6)
                        | num.at((y, x)).eq(8),
                );
            }
            solver.add_expr(count_true(is_even).eq(4));
        }
    }

    solver.irrefutable_facts().map(|f| f.get(num))
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
    problem_to_url(combinator(), "sudoku2", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["sudoku", "sudoku2"], url)
}
