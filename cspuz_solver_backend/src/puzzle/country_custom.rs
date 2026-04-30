use crate::board::{Board, BoardKind, Item, ItemKind};
use crate::uniqueness::check_uniqueness;
use cspuz_rs::graph;
use cspuz_rs_puzzles::puzzles::country_custom;

pub fn solve(url: &str) -> Result<Board, &'static str> {
    let (empty, (borders, clues)) = country_custom::deserialize_problem(url).ok_or("invalid url")?;
    let is_line = country_custom::solve_country_custom(empty, &borders, &clues);

    let height = borders.vertical.len();
    let width = borders.vertical[0].len() + 1;
    let mut board = Board::new(BoardKind::Grid, height, width, check_uniqueness(&is_line));
    board.add_borders(&borders, "black");

    let rooms = graph::borders_to_rooms(&borders);
    assert_eq!(rooms.len(), clues.len());
    for i in 0..rooms.len() {
        if let Some(n) = clues[i] {
            let (y, x) = rooms[i][0];
            if n >= 0 {
                board.push(Item::cell(y, x, "black", ItemKind::Num(n)));
            } else {
                board.push(Item::cell(y, x, "black", ItemKind::Text("?")));
            }
        }
    }
    if let Some(is_line) = is_line {
        board.add_lines_irrefutable_facts(&is_line, "green", None);
    }

    Ok(board)
}
