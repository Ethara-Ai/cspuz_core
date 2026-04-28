// yajilin_custom.rs — Yajilin with ALL 3 custom rules always enabled:
// R6: No diagonal shading (no two diagonally adjacent cells may both be shaded)
// R7: Max shaded per row/column = ceil(min(h,w) / 2)
// R8: Shaded outside only (shaded cells cannot be inside the loop)

use crate::puzzles::loop_common::force_shaded_outside;
use crate::util;
use cspuz_rs::graph;
use cspuz_rs::items::NumberedArrow;
use cspuz_rs::serializer::{
    url_to_problem, Choice, Combinator, Dict, Grid, MaybeSkip, NumberedArrowCombinator,
    Optionalize, Spaces, Tuple2,
};
use cspuz_rs::solver::Solver;

pub fn solve_yajilin_custom(
    clues: &[Vec<Option<NumberedArrow>>],
) -> Option<(graph::BoolGridEdgesIrrefutableFacts, Vec<Vec<Option<bool>>>)> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_line = &graph::BoolGridEdges::new(&mut solver, (h - 1, w - 1));
    solver.add_answer_key_bool(&is_line.horizontal);
    solver.add_answer_key_bool(&is_line.vertical);

    let is_passed = &graph::single_cycle_grid_edges(&mut solver, is_line);
    let is_black = &solver.bool_var_2d((h, w));

    // === R8: Force shaded outside (always enabled) ===
    force_shaded_outside(&mut solver, is_black, is_line, h, w);

    solver.add_answer_key_bool(is_black);

    // Standard: no orthogonal adjacent shading
    solver.add_expr(!is_black.conv2d_and((1, 2)));
    solver.add_expr(!is_black.conv2d_and((2, 1)));

    // === R6: No diagonal adjacent shading ===
    solver.add_expr(!is_black.conv2d_and((2, 2)));

    // === R7: Max shaded per row/column ===
    let min_dim = h.min(w);
    let max_shaded = ((min_dim as i32) + 1) / 2; // ceil(min(h,w) / 2)
    for y in 0..h {
        solver.add_expr(is_black.slice_fixed_y((y, ..)).count_true().le(max_shaded));
    }
    for x in 0..w {
        solver.add_expr(is_black.slice_fixed_x((.., x)).count_true().le(max_shaded));
    }

    // Standard: clue cells are not on loop and not shaded; arrow counting
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
    url_to_problem(combinator(), &["yajilin", "yajirin", "yajilin_custom"], url)
}
