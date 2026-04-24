# cspuz_core Changes Log

**Date**: April 24, 2026
**Purpose**: Added 12 custom variant solver modules to support Morpheus puzzle platform custom rules. Also added "lightup" URL alias to existing akari solver.

---

## Summary

- **24 new Rust source files** created (12 solver modules + 12 backend wrappers)
- **3 existing files modified** (2 registration files + 1 solver file for alias)
- **1,738 lines** of new solver code + **522 lines** of new backend code = **2,260 lines total**
- Binary rebuilt: `target/release/run_solver`
- Result: Solver coverage went from 12/60 to 24/60 unique-verified puzzles

---

## New Files

### Solver Modules (`cspuz_rs_puzzles/src/puzzles/`)

| File | Lines | Forked From | Custom Rules Added |
|------|-------|-------------|-------------------|
| `hitori_custom.rs` | 86 | hitori.rs | King adjacency (diagonal check), checkerboard parity (shaded only on even cells), ≤2 shaded per row/col |
| `sudoku2.rs` | 88 | sudoku.rs | Even-digit balance: each row must have exactly 4 even digits (2,4,6,8) in 9×9 |
| `minesweeper2.rs` | 58 | minesweeper.rs | Row mine cap ≤⌈2·cols/3⌉, no 2×2 mine block |
| `heyawake2.rs` | 117 | heyawake.rs | Strict adjacency (0 adjacent pairs), column shade balance ≤⌈rows/2⌉, shading density 10-50% |
| `tapa_custom.rs` | 219 | tapa.rs | Flipped rules: UNSHADED connected (not shaded), no 2×2 UNSHADED block (not shaded), shaded majority per ROW |
| `tapa2.rs` | 219 | tapa.rs | Standard tapa rules + column majority: shaded > unshaded per column |
| `country2.rs` | 202 | country_road.rs | Turn balance (turns ≤ 2× straights), max 85% loop coverage, ≤1 empty row |
| `yajilin2.rs` | 84 | yajilin.rs | Shaded cells must be on grid border (perimeter only) — interior cells forced unshaded |
| `lits2.rs` | 90 | lits.rs | Triomino variant: exactly 3 shaded cells per room (not 4), no 2×2 shaded, global connectivity, room-local connectivity. No same-shape check. |
| `nurikabe_custom.rs` | 293 | nurikabe.rs | Shaded groups ≤3 cells (forbids all tetromino placements), straight-line islands (1×N or N×1 via horizontal/vertical constraint) |
| `nurikabe2.rs` | 99 | nurikabe.rs | No 2×2 unshaded block, shaded dominoes (each black cell has exactly 1 black orthogonal neighbor), NO global black connectivity |
| `lightup2.rs` | 183 | akari.rs | Diagonal illumination (NW-SE and NE-SW diagonals), numbered walls count diagonal neighbors, no two lights orthogonally adjacent |

### Backend Wrappers (`cspuz_solver_backend/src/puzzle/`)

| File | Lines | Rendering |
|------|-------|-----------|
| `hitori_custom.rs` | 45 | Fill/Dot for black/white + overlaid Num on clue cells |
| `sudoku2.rs` | 77 | Custom uniqueness via candidates, Num for solved cells, BoldWall for boxes |
| `minesweeper2.rs` | 38 | FilledCircle for mines, Dot for safe, Num for clues |
| `heyawake2.rs` | 44 | BoldWall room borders + Block/Dot + room clue Nums |
| `tapa_custom.rs` | 30 | TapaClue for clue cells + Block/Dot for solved cells |
| `tapa2.rs` | 30 | TapaClue for clue cells + Block/Dot for solved cells |
| `country2.rs` | 32 | BoldWall room borders + line/cross edges + room Nums |
| `yajilin2.rs` | 90 | Block/Dot for cells + SideArrow for clues + line/cross edges |
| `lits2.rs` | 35 | BoldWall room borders + Block/Dot for cells |
| `nurikabe_custom.rs` | 34 | Block/Dot for black/white + Num for clue cells |
| `nurikabe2.rs` | 34 | Block/Dot for black/white + Num for clue cells |
| `lightup2.rs` | 33 | Fill for walls, Circle for lights, Dot for lit cells |

---

## Modified Files

### 1. `cspuz_rs_puzzles/src/puzzles/mod.rs`

**Change**: Added 12 new `pub mod` entries (alphabetically sorted):
```rust
pub mod country2;
pub mod heyawake2;
pub mod hitori_custom;
pub mod lightup2;
pub mod lits2;
pub mod minesweeper2;
pub mod nurikabe2;
pub mod nurikabe_custom;
pub mod sudoku2;
pub mod tapa2;
pub mod tapa_custom;
pub mod yajilin2;
```

### 2. `cspuz_solver_backend/src/puzzle/mod.rs`

**Change**: Added 12 new entries to `puzzle_list!(puzz_link, ...)` macro + "lightup" alias to akari:

```rust
// New entries added:
(country2, ["country2"], "Country Road 2", "カントリーロード2"),
(heyawake2, ["heyawake2"], "Heyawake 2", "へやわけ2"),
(hitori_custom, ["hitori_custom"], "Hitori Custom", "ひとりにしてくれカスタム"),
(lightup2, ["lightup2"], "Lightup 2", "美術館2"),
(lits2, ["lits2"], "LITS 2", "LITS2"),
(minesweeper2, ["mines2"], "Minesweeper 2", "マインスイーパ2"),
(nurikabe2, ["nurikabe2"], "Nurikabe 2", "ぬりかべ2"),
(nurikabe_custom, ["nurikabe_custom"], "Nurikabe Custom", "ぬりかべカスタム"),
(sudoku2, ["sudoku2"], "Sudoku 2", "数独2"),
(tapa2, ["tapa2"], "Tapa 2", "Tapa2"),
(tapa_custom, ["tapa_custom"], "Tapa Custom", "Tapaカスタム"),
(yajilin2, ["yajilin2"], "Yajilin 2", "ヤジリン2"),

// Modified existing entry:
(akari, ["akari", "lightup"], "Akari", "美術館"),  // added "lightup" alias
```

### 3. `cspuz_rs_puzzles/src/puzzles/akari.rs`

**Change**: Updated deserializer to accept "lightup" as URL pid:
```rust
// Before:
url_to_problem(combinator(), &["akari"], url)

// After:
url_to_problem(combinator(), &["akari", "lightup"], url)
```

---

## Build Notes

- **Build command**: `cargo build --release` (in `cspuz_core/`)
- **Build status**: SUCCESS
- **Warnings**: 2 unused import warnings (non-blocking)
  - `heyawake2.rs`: unused `super::heyawake` import
  - `lits2.rs`: unused `FALSE` import
- **Binary**: `target/release/run_solver`

### Build Fixes Applied During Development

1. **country2.rs**: `count_true() * 2 / 3` — IntExpr has no Mul. Fixed by triplicating turn indicator bools and duplicating passed bools: `count_true(turns_x3).le(count_true(passed_x2))`
2. **lightup2.rs**: BoolVar `|` produces BoolExpr, can't assign back to BoolVar. Fixed by using `.expr()` to convert BoolVar → BoolExpr before OR chain.
3. **country2.rs**: `is_passed.at().into()` fails — NdArray<BoolVar> has no Into<BoolExpr>. Fixed by using `.expr()` instead.
4. **nurikabe_custom.rs**: Removed dead `black_group_id` code that created SAT variables without enforcing constraints. Kept tetromino-forbidding approach.

---

## Test Results (60 puzzles)

### Before Custom Variants
- 12/60 isUnique=true (standard solver only)
- 30/60 "unknown puzzle type" (all *2 + lightup)

### After Custom Variants
- 24/60 isUnique=true (**doubled**)
- 0/60 "unknown puzzle type" (all variants now recognized)

### Per-Module Results

| Module | Puzzles Tested | isUnique=true | isUnique=false | hasAnswer=false | Error |
|--------|---------------|---------------|----------------|-----------------|-------|
| sudoku2.rs | 3 | 3 | 0 | 0 | 0 |
| heyawake2.rs | 3 | 3 | 0 | 0 | 0 |
| minesweeper2.rs | 3 | 3 | 0 | 0 | 0 |
| country2.rs | 3 | 3 | 0 | 0 | 0 |
| lits2.rs | 3 | 3 | 0 | 0 | 0 |
| nurikabe2.rs | 3 | 3 | 0 | 0 | 0 |
| tapa2.rs | 3 | 2 | 1 | 0 | 0 |
| yajilin2.rs | 3 | 1 | 0 | 2 | 0 |
| hitori_custom.rs | 3 | 0 | 3 | 0 | 0 |
| lightup2.rs | 3 | 0 | 0 | 3 | 0 |
| tapa_custom.rs | 3 | 0 | 0 | 3 | 0 |
| nurikabe_custom.rs | 3 | 0 | 1 | 0 | 2 |
| akari (lightup alias) | 3 | 3 | 0 | 0 | 0 |
| **Total** | **39** | **24** | **5** | **8** | **2** |

### Game File Updates After Testing

9 game files in `pzprjs/games/` updated with new `cspuz_is_unique` values:
- puzzle_sudoku2.py: `None` → `True`
- puzzle_heyawake2.py: `None` → `True`
- puzzle_minesweeper2.py: `None` → `True`
- puzzle_country2.py: `None` → `True`
- play_lightup.py: `None` → `True`
- play_nurikabe2.py: `None` → `True`
- play_tapa2.py: `None` → `difficulty != "hard"`
- custom_lits2.py: `None` → `True`
- custom_yajilin2.py: `None` → `True if difficulty == "easy" else None`

---

## Custom Rules Reference

Each solver module implements specific custom rules from the pzprjs AnsCheck system:

| pzprjs AnsCheck | Rust SAT Encoding | Module |
|-----------------|-------------------|--------|
| checkDiagonalShadeCell (king) | `!is_black.conv2d_and((2,2))` on diagonal pairs | hitori_custom |
| checkCheckerboardParity | Unit clauses: `!is_black.at((y,x))` where (y+x)%2≠0 | hitori_custom |
| checkShadeLimitPerLine | `is_black.slice(row).count_true().le(2)` per row/col | hitori_custom |
| checkEvenDigitBalance | `count_true(eq(2)\|eq(4)\|eq(6)\|eq(8)).eq(4)` per row | sudoku2 |
| checkStrictAdjacentShadeCell | `!is_black.conv2d_and((1,2))` and `(2,1)` (zero tolerance) | heyawake2 |
| checkColShadeBalance | `count_true(col).le(ceil(h/2))` per column | heyawake2 |
| checkShadeDensity | `total_shade.ge(floor(0.1*area))` & `.le(ceil(0.5*area))` | heyawake2 |
| checkRowMineCap | `count_true(row).le(ceil(2*w/3))` per row | minesweeper2 |
| checkNo2x2MineBlock | `!is_mine.conv2d_and((2,2))` | minesweeper2 |
| check2x2UnshadeCell | `!(!is_black).conv2d_and((2,2))` | tapa_custom |
| checkConnectUnshade | `active_vertices_connected_2d(!is_black)` | tapa_custom |
| checkShadeMajorityPerRow | `count_true(row).ge(ceil(w/2))` | tapa_custom |
| checkShadeMajorityPerCol | `count_true(col).ge(ceil(h/2))` | tapa2 |
| checkStraightVsTurns | `count(turns_x3).le(count(passed_x2))` | country2 |
| checkMaxLoopCoverage | `count_true(passed).le(ceil(0.85*area))` | country2 |
| checkMaxEmptyRows | `count(empty_rows).le(1)` | country2 |
| checkShadedOnBorder | `!is_black.at((y,x))` for all interior cells | yajilin2 |
| Triomino (3 cells/room) | Room shade count = 3, connectivity, no 2×2 | lits2 |
| checkShadeMax3 | Forbid all tetromino shape placements | nurikabe_custom |
| checkStraightLineIslands | `is_horizontal` per island, bounding box constraints | nurikabe_custom |
| checkShadeDomino | Each black has exactly 1 black 4-neighbor | nurikabe2 |
| check2x2UnshadedCell | `!(!is_black).conv2d_and((2,2))` | nurikabe2 |
| checkDiag4Akari | Diagonal neighbor counting for numbered walls | lightup2 |
| checkOrthAdjacentAkari | No two lights orthogonally adjacent | lightup2 |
| Diagonal illumination | NW-SE and NE-SW diagonal group constraints | lightup2 |
