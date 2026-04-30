/// Custom Heyawake with relaxed adjacency (at most 1 adjacent pair) + row shade balance
use cspuz_rs::graph;
use cspuz_rs::serializer::{
    problem_to_url_with_context, url_to_problem, Choice, Combinator, Context, Dict, HexInt,
    Optionalize, RoomsWithValues, Size, Spaces,
};
use cspuz_rs::solver::{count_true, Solver};

pub fn solve_heyawake_custom(
    borders: &graph::InnerGridEdges<Vec<Vec<bool>>>,
    clues: &[Option<i32>],
) -> Option<Vec<Vec<Option<bool>>>> {
    let (h, w) = borders.base_shape();

    let mut solver = Solver::new();
    let is_black = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_black);

    graph::active_vertices_connected_2d(&mut solver, !is_black);

    // CUSTOM: Relaxed adjacency — at most 1 adjacent pair TOTAL (H + V combined)
    // pzprjs checkAdjacentShadeCell@heyawake counts ALL pairs and fails if pairCount > 1
    {
        let mut all_adj_pairs = vec![];
        for y in 0..h {
            for x in 0..(w - 1) {
                all_adj_pairs.push(is_black.at((y, x)) & is_black.at((y, x + 1)));
            }
        }
        for y in 0..(h - 1) {
            for x in 0..w {
                all_adj_pairs.push(is_black.at((y, x)) & is_black.at((y + 1, x)));
            }
        }
        solver.add_expr(count_true(all_adj_pairs).le(1));
    }

    // Line-of-sight constraint
    for y in 0..h {
        for x in 0..w {
            if y + 2 < h && borders.horizontal[y][x] {
                let mut y2 = y + 2;
                while y2 < h && !borders.horizontal[y2 - 1][x] {
                    y2 += 1;
                }
                if y2 < h {
                    solver.add_expr(is_black.slice_fixed_x((y..=y2, x)).any());
                }
            }
            if x + 2 < w && borders.vertical[y][x] {
                let mut x2 = x + 2;
                while x2 < w && !borders.vertical[y][x2 - 1] {
                    x2 += 1;
                }
                if x2 < w {
                    solver.add_expr(is_black.slice_fixed_y((y, x..=x2)).any());
                }
            }
        }
    }

    let rooms = graph::borders_to_rooms(borders);
    assert_eq!(rooms.len(), clues.len());
    for i in 0..rooms.len() {
        if let Some(n) = clues[i] {
            let mut cells = vec![];
            for &pt in &rooms[i] {
                cells.push(is_black.at(pt));
            }
            if n >= 0 {
                solver.add_expr(count_true(cells).eq(n));
            }
        }
    }

    // CUSTOM: Row shade balance — at most ceil(cols/2) shaded per row
    let row_cap = ((w + 1) / 2) as i32;
    for y in 0..h {
        solver.add_expr(is_black.slice_fixed_y((y, ..)).count_true().le(row_cap));
    }

    solver.irrefutable_facts().map(|f| f.get(is_black))
}

pub(super) type Problem = (graph::InnerGridEdges<Vec<Vec<bool>>>, Vec<Option<i32>>);

fn combinator() -> impl Combinator<Problem> {
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
        "heyawake",
        problem.clone(),
        &Context::sized(height, width),
    )
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["heyawake", "heyawake_custom"], url)
}
