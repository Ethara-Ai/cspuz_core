/// Custom Country Road with >= 50% coverage + <= 1 empty row
use cspuz_rs::graph;
use cspuz_rs::serializer::{
    url_to_problem, Choice, Combinator, Dict, HexInt, Optionalize, RoomsWithValues, Size, Spaces,
    Tuple2,
};
use cspuz_rs::solver::{count_true, Solver};

pub fn solve_country_custom(
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

    // CUSTOM: Min 50% loop coverage
    let total = (h * w) as i32;
    let min_on_loop = (total + 1) / 2;
    {
        let mut all_passed = vec![];
        for y in 0..h {
            for x in 0..w {
                all_passed.push(is_passed.at((y, x)));
            }
        }
        solver.add_expr(count_true(&all_passed).ge(min_on_loop));
    }

    // CUSTOM: Max 1 empty row
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
    url_to_problem(combinator(), &["country", "country_custom"], url)
}
