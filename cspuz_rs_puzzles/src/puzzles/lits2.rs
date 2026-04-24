// lits2.rs — LITS variant using triominoes (3 cells per room) instead of tetrominoes (4)
// No same-shape adjacency check (unlike standard LITS)
// Same URL format as lits, alias "lits2"

use cspuz_rs::graph;
use cspuz_rs::serializer::{url_to_problem, Combinator, Rooms, Size};
use cspuz_rs::solver::{count_true, Solver, FALSE};

pub fn solve_lits2(
    borders: &graph::InnerGridEdges<Vec<Vec<bool>>>,
) -> Option<Vec<Vec<Option<bool>>>> {
    let h = borders.vertical.len();
    assert!(h > 0);
    let w = borders.vertical[0].len() + 1;

    let mut solver = Solver::new();
    let is_black = &solver.bool_var_2d((h, w));
    solver.add_answer_key_bool(is_black);

    // Global shaded connectivity
    graph::active_vertices_connected_2d(&mut solver, is_black);

    // No 2×2 shaded block
    solver.add_expr(
        !(is_black.slice((..(h - 1), ..(w - 1)))
            & is_black.slice((..(h - 1), 1..))
            & is_black.slice((1.., ..(w - 1)))
            & is_black.slice((1.., 1..))),
    );

    let rooms = graph::borders_to_rooms(borders);

    for room in rooms.iter() {
        // Exactly 3 shaded cells per room (triomino)
        let mut room_cells = vec![];
        for &(y, x) in room {
            room_cells.push(is_black.at((y, x)));
        }
        solver.add_expr(count_true(&room_cells).eq(3));

        // Shaded cells in room must be connected (within room)
        // Build room-local adjacency: for each pair of room cells that are orthogonal neighbors
        // and NOT separated by a border, they share connectivity
        // Since we're only looking within a room, borders within the room are all absent
        // We just need the shaded cells in the room to form a connected group
        // Use the approach: for each shaded cell in the room, it must have at least one
        // shaded neighbor within the room (except if room has only 1 shaded — but we have 3)

        // Build neighbor list within room for connectivity
        let room_set: std::collections::HashSet<(usize, usize)> = room.iter().cloned().collect();
        for &(y, x) in room {
            let mut neighbors_in_room = vec![];
            if y > 0 && room_set.contains(&(y - 1, x)) && !borders.horizontal[y - 1][x] {
                neighbors_in_room.push(is_black.at((y - 1, x)).expr());
            }
            if x > 0 && room_set.contains(&(y, x - 1)) && !borders.vertical[y][x - 1] {
                neighbors_in_room.push(is_black.at((y, x - 1)).expr());
            }
            if y < h - 1 && room_set.contains(&(y + 1, x)) && !borders.horizontal[y][x] {
                neighbors_in_room.push(is_black.at((y + 1, x)).expr());
            }
            if x < w - 1 && room_set.contains(&(y, x + 1)) && !borders.vertical[y][x] {
                neighbors_in_room.push(is_black.at((y, x + 1)).expr());
            }

            // If this cell is shaded, at least one room neighbor must also be shaded
            if neighbors_in_room.is_empty() {
                // No neighbors in room — cell can't be shaded
                solver.add_expr(!is_black.at((y, x)));
            } else {
                use cspuz_rs::solver::any;
                solver.add_expr(is_black.at((y, x)).imp(any(neighbors_in_room)));
            }
        }
    }

    // No same-shape check for lits2 (unlike standard LITS)

    solver.irrefutable_facts().map(|f| f.get(is_black))
}

type Problem = graph::InnerGridEdges<Vec<Vec<bool>>>;

fn combinator() -> impl Combinator<Problem> {
    Size::new(Rooms)
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["lits", "lits2"], url)
}
