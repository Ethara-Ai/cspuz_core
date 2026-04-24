// Custom Nurikabe2 variant:
// - check2x2UnshadedCell: No 2×2 block of unshaded (white) cells
// - checkShadeDomino: Every shaded (black) connected region must be exactly 2 cells (a domino)
// - Standard: island sizes match clues, each island connected
// - NO global black connectivity requirement
// - NO no-2×2 black constraint (standard nurikabe has this, nurikabe2 does not)

use crate::util;
use cspuz_rs::graph;
use cspuz_rs::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, HexInt, Optionalize, Spaces,
};
use cspuz_rs::solver::Solver;

pub fn solve_nurikabe2(clues: &[Vec<Option<i32>>]) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_black = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_black);

    let mut clue_pos = vec![];
    for y in 0..h {
        for x in 0..w {
            if let Some(n) = clues[y][x] {
                clue_pos.push((y, x, n));
            }
        }
    }

    // Group IDs: 0 = black, 1..=N = island groups
    let group_id = solver.int_var_2d((h, w), 0, clue_pos.len() as i32);
    solver.add_expr(is_black.iff(group_id.eq(0)));

    // Each island connected
    for i in 1..=clue_pos.len() {
        graph::active_vertices_connected_2d(&mut solver, group_id.eq(i as i32));
    }

    // Adjacent unshaded cells must share same group
    solver.add_expr(
        (!is_black.conv2d_or((2, 1))).imp(
            group_id
                .slice((..(h - 1), ..))
                .eq(group_id.slice((1.., ..))),
        ),
    );
    solver.add_expr(
        (!is_black.conv2d_or((1, 2))).imp(
            group_id
                .slice((.., ..(w - 1)))
                .eq(group_id.slice((.., 1..))),
        ),
    );

    // Clue cells must be in their group with correct size
    for (i, &(y, x, n)) in clue_pos.iter().enumerate() {
        solver.add_expr(group_id.at((y, x)).eq((i + 1) as i32));
        if n > 0 {
            solver.add_expr(group_id.eq((i + 1) as i32).count_true().eq(n));
        }
    }

    // CUSTOM: No 2×2 unshaded block
    solver.add_expr(!(!is_black).conv2d_and((2, 2)));

    // CUSTOM: Every black connected region must be exactly 2 cells (domino)
    // Approach: Each black cell must have EXACTLY 1 black orthogonal neighbor
    for y in 0..h {
        for x in 0..w {
            let neighbors = is_black.four_neighbors((y, x));
            // If this cell is black, exactly 1 of its neighbors is black
            solver.add_expr(is_black.at((y, x)).imp(neighbors.count_true().eq(1)));
        }
    }

    // NO global black connectivity (explicitly not required for nurikabe2)
    // NO no-2×2 black constraint

    solver.irrefutable_facts().map(|f| f.get(is_black))
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
    problem_to_url(combinator(), "nurikabe2", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["nurikabe", "nurikabe2"], url)
}
