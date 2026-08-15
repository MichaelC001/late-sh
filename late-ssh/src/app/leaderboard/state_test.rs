use late_core::models::leaderboard::{DailyPuzzle, DoorGame, ScoreGame};

use crate::app::leaderboard::state::*;

/// The page list is roster-derived: each door's board triple leads, then the
/// two bespoke boards, then the Lateania boards, then every daily puzzle,
/// then every score game. A roster addition in late-core must grow this list
/// without any page change.
#[test]
fn board_list_follows_the_rosters() {
    let boards = Board::all();
    assert_eq!(boards[0], Board::DoorWins(DoorGame::ALL[0]));
    assert_eq!(boards[1], Board::DoorDepth(DoorGame::ALL[0]));
    assert_eq!(boards[2], Board::DoorScore(DoorGame::ALL[0]));
    let door_boards = 3 * DoorGame::ALL.len();
    assert_eq!(boards[door_boards], Board::TopChips);
    assert_eq!(boards[door_boards + 1], Board::ArcadeWins);
    assert_eq!(boards[door_boards + 2], Board::LateaniaAdventurers);
    assert_eq!(boards[door_boards + 3], Board::LateaniaFrontier);
    assert_eq!(
        boards.len(),
        4 + door_boards + DailyPuzzle::ALL.len() + ScoreGame::ALL.len()
    );
    assert_eq!(boards[4 + door_boards], Board::Daily(DailyPuzzle::ALL[0]));
    assert_eq!(
        boards[4 + door_boards + DailyPuzzle::ALL.len()],
        Board::Score(ScoreGame::ALL[0])
    );
}

#[test]
fn selection_wraps_both_ways() {
    let mut state = LeaderboardPageState::new();
    assert_eq!(state.selected_board(), Board::DoorWins(DoorGame::ALL[0]));
    state.select_previous();
    assert_eq!(
        state.selected_index(),
        state.boards().len() - 1,
        "previous from the first board wraps to the last"
    );
    state.select_next();
    assert_eq!(state.selected_board(), Board::DoorWins(DoorGame::ALL[0]));
}
