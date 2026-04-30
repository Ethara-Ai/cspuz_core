/// Custom Sudoku variant with additional constraint:
/// - Killer cage sum: every 3x3 box sums to 45
/// Only applies to 9x9 sudoku.
use crate::util;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, HexInt, Optionalize, Spaces,
};
use cspuz_rs::solver::Solver;

pub fn solve_sudoku_custom(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<i32>>>> {
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

    // CUSTOM: Killer cage sum — every 3x3 box sums to 45 (for 9x9)
    if n == 9 {
        for bi in 0..bw {
            for bj in 0..bh {
                let mut box_sum = num.at((bi * bh, bj * bw)).expr();
                for y in (bi * bh)..((bi + 1) * bh) {
                    for x in (bj * bw)..((bj + 1) * bw) {
                        if y == bi * bh && x == bj * bw {
                            continue;
                        }
                        box_sum = box_sum + num.at((y, x));
                    }
                }
                solver.add_expr(box_sum.eq(45));
            }
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
    problem_to_url(combinator(), "sudoku", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["sudoku", "sudoku_custom"], url)
}
