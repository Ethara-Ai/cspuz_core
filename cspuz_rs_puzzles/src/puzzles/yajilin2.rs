// yajilin2.rs — Yajilin variant with custom rule:
// - Shaded cells must be on the grid border (perimeter only)
// Same URL format as yajilin, alias "yajilin2"

use crate::util;
use cspuz_rs::graph;
use cspuz_rs::items::NumberedArrow;
use cspuz_rs::serializer::{
    url_to_problem, Choice, Combinator, Dict, Grid, MaybeSkip, NumberedArrowCombinator,
    Optionalize, Spaces, Tuple2,
};
use cspuz_rs::solver::Solver;

pub fn solve_yajilin2(
    outside: bool,
    clues: &[Vec<Option<NumberedArrow>>],
) -> Option<(graph::BoolGridEdgesIrrefutableFacts, Vec<Vec<Option<bool>>>)> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_line = &graph::BoolGridEdges::new(&mut solver, (h - 1, w - 1));
    solver.add_answer_key_bool(&is_line.horizontal);
    solver.add_answer_key_bool(&is_line.vertical);

    let is_passed = &graph::single_cycle_grid_edges(&mut solver, is_line);
    let is_black = &solver.bool_var_2d((h, w));
    if outside {
        crate::puzzles::loop_common::force_shaded_outside(&mut solver, is_black, is_line, h, w);
    }
    solver.add_answer_key_bool(is_black);
    solver.add_expr(!is_black.conv2d_and((1, 2)));
    solver.add_expr(!is_black.conv2d_and((2, 1)));

    for y in 0..h {
        for x in 0..w {
            if let Some((dir, n)) = clues[y][x] {
                solver.add_expr(!is_passed.at((y, x)));
                solver.add_expr(!is_black.at((y, x)));
                if n < 0 {
                    continue;
                }
                if let Some(cells) = is_black.pointing_cells((y, x), dir) {
                    solver.add_expr(cells.count_true().eq(n));
                }
            } else {
                solver.add_expr(is_passed.at((y, x)) ^ is_black.at((y, x)));
            }
        }
    }

    // === CUSTOM RULE: Shaded cells must be on the grid border ===
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            solver.add_expr(!is_black.at((y, x)));
        }
    }

    solver
        .irrefutable_facts()
        .map(|f| (f.get(is_line), f.get(is_black)))
}

type Problem = (bool, Vec<Vec<Option<NumberedArrow>>>);

fn combinator() -> impl Combinator<Problem> {
    Tuple2::new(
        Choice::new(vec![
            Box::new(Dict::new(true, "o/")),
            Box::new(Dict::new(true, "ob/")),
            Box::new(Dict::new(false, "")),
        ]),
        MaybeSkip::new(
            "b/",
            Grid::new(Choice::new(vec![
                Box::new(Optionalize::new(NumberedArrowCombinator)),
                Box::new(Spaces::new(None, 'a')),
            ])),
        ),
    )
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["yajilin", "yajirin", "yajilin2"], url)
}
