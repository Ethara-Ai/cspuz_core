// country2.rs — Country Road variant with custom rules:
// - Max 85% loop coverage: onLoop <= ceil(total * 0.85)
// - Turn balance: turnCount <= straightCount * 2
// - Max 1 empty row (rows with no loop cells)
// Same URL format as country road, alias "country2"

use cspuz_rs::graph;
use cspuz_rs::serializer::{
    url_to_problem, Choice, Combinator, Dict, HexInt, Optionalize, RoomsWithValues, Size, Spaces,
    Tuple2,
};
use cspuz_rs::solver::{count_true, Solver};

pub fn solve_country2(
    empty: bool,
    borders: &graph::InnerGridEdges<Vec<Vec<bool>>>,
    clues: &[Option<i32>],
) -> Option<graph::BoolGridEdgesIrrefutableFacts> {
    let (h, w) = borders.base_shape();

    let mut solver = Solver::new();
    let rooms = graph::borders_to_rooms(borders);
    let is_line = &graph::BoolGridEdges::new(&mut solver, (h - 1, w - 1));
    solver.add_answer_key_bool(&is_line.horizontal);
    solver.add_answer_key_bool(&is_line.vertical);

    let is_passed = &graph::single_cycle_grid_edges(&mut solver, is_line);
    let mut room_id = vec![vec![0; w]; h];

    for i in 0..rooms.len() {
        for &(y, x) in &rooms[i] {
            room_id[y][x] = i;
        }
    }

    let mut room_entrance = vec![vec![]; rooms.len()];
    for y in 0..h {
        for x in 0..w {
            if y < h - 1 && room_id[y][x] != room_id[y + 1][x] {
                room_entrance[room_id[y][x]].push(is_line.vertical.at((y, x)));
                room_entrance[room_id[y + 1][x]].push(is_line.vertical.at((y, x)));
            }
            if x < w - 1 && room_id[y][x] != room_id[y][x + 1] {
                room_entrance[room_id[y][x]].push(is_line.horizontal.at((y, x)));
                room_entrance[room_id[y][x + 1]].push(is_line.horizontal.at((y, x)));
            }
        }
    }

    for i in 0..rooms.len() {
        if empty {
            solver.add_expr(
                count_true(&room_entrance[i]).eq(0) | count_true(&room_entrance[i]).eq(2),
            );
        } else {
            solver.add_expr(count_true(&room_entrance[i]).eq(2));
        }
    }

    for y in 0..h {
        for x in 0..w {
            if y < h - 1 && borders.horizontal[y][x] {
                solver.add_expr(is_passed.at((y, x)) | is_passed.at((y + 1, x)));
            }
            if x < w - 1 && borders.vertical[y][x] {
                solver.add_expr(is_passed.at((y, x)) | is_passed.at((y, x + 1)));
            }
        }
    }

    for i in 0..rooms.len() {
        if let Some(n) = clues[i] {
            let mut cells = vec![];
            for &pt in &rooms[i] {
                cells.push(is_passed.at(pt));
            }
            if n >= 0 {
                solver.add_expr(count_true(cells).eq(n));
            }
        }
    }

    // === CUSTOM RULES ===

    // Rule: Max 85% loop coverage — onLoop <= ceil(total * 0.85)
    let total = h * w;
    let max_on_loop = ((total as f64) * 0.85).ceil() as i32;
    {
        let mut all_passed = vec![];
        for y in 0..h {
            for x in 0..w {
                all_passed.push(is_passed.at((y, x)));
            }
        }
        solver.add_expr(count_true(&all_passed).le(max_on_loop));
    }

    // Rule: Turn balance — turnCount <= straightCount * 2
    // A cell on the loop has exactly 2 line segments touching it.
    // It's a "turn" if those 2 segments are perpendicular (one H, one V).
    // It's a "straight" if both are same direction (both H or both V).
    // For each on-loop cell: count H neighbors on line, count V neighbors on line.
    // Turn iff h_count == 1 && v_count == 1.
    // Straight iff h_count == 2 || v_count == 2.
    // turnCount <= 2 * straightCount ⟺ turnCount <= 2 * (onLoop - turnCount) ⟺ 3*turnCount <= 2*onLoop
    {
        let mut turn_indicators = vec![];
        for y in 0..h {
            for x in 0..w {
                // Count horizontal line segments adjacent to this cell
                let mut h_segs = vec![];
                if x > 0 {
                    h_segs.push(is_line.horizontal.at((y, x - 1)));
                }
                if x < w - 1 {
                    h_segs.push(is_line.horizontal.at((y, x)));
                }
                // Count vertical line segments adjacent to this cell
                let mut v_segs = vec![];
                if y > 0 {
                    v_segs.push(is_line.vertical.at((y - 1, x)));
                }
                if y < h - 1 {
                    v_segs.push(is_line.vertical.at((y, x)));
                }
                // Turn = on loop AND has exactly 1 horizontal and 1 vertical
                // = is_passed AND count_h == 1 AND count_v == 1
                let is_turn = &solver.bool_var();
                solver.add_expr(is_turn.imp(
                    is_passed.at((y, x)) & count_true(&h_segs).eq(1) & count_true(&v_segs).eq(1),
                ));
                solver.add_expr(
                    (is_passed.at((y, x)) & count_true(&h_segs).eq(1) & count_true(&v_segs).eq(1))
                        .imp(is_turn),
                );
                turn_indicators.push(is_turn.expr());
            }
        }
        // 3 * turnCount <= 2 * onLoop  ↔  count(3 copies of turns) <= count(2 copies of passed)
        let mut turns_x3: Vec<cspuz_rs::solver::BoolExpr> = vec![];
        for t in &turn_indicators {
            turns_x3.push(t.clone());
            turns_x3.push(t.clone());
            turns_x3.push(t.clone());
        }
        let mut passed_x2 = vec![];
        for y in 0..h {
            for x in 0..w {
                let p = is_passed.at((y, x)).expr();
                passed_x2.push(p.clone());
                passed_x2.push(p.clone());
            }
        }
        solver.add_expr(count_true(&turns_x3).le(count_true(&passed_x2)));
    }

    // Rule: Max 1 empty row — at most 1 row where no cell is on the loop
    {
        let mut empty_row_indicators = vec![];
        for y in 0..h {
            let mut row_cells = vec![];
            for x in 0..w {
                row_cells.push(is_passed.at((y, x)));
            }
            let is_empty_row = &solver.bool_var();
            solver.add_expr(is_empty_row.iff(count_true(&row_cells).eq(0)));
            empty_row_indicators.push(is_empty_row.expr());
        }
        solver.add_expr(count_true(&empty_row_indicators).le(1));
    }

    solver.irrefutable_facts().map(|f| f.get(is_line))
}

type Problem = (
    bool,
    (graph::InnerGridEdges<Vec<Vec<bool>>>, Vec<Option<i32>>),
);

fn combinator() -> impl Combinator<Problem> {
    Tuple2::new(
        Choice::new(vec![
            Box::new(Dict::new(true, "e/")),
            Box::new(Dict::new(false, "")),
        ]),
        Size::new(RoomsWithValues::new(Choice::new(vec![
            Box::new(Optionalize::new(HexInt)),
            Box::new(Spaces::new(None, 'g')),
            Box::new(Dict::new(Some(-1), ".")),
        ]))),
    )
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["country", "country2"], url)
}
