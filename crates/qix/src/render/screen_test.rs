use qix_core::tuning::{
    CLAIM_THRESHOLD_RANGE, HIGH_SCORE_COUNT, MARKER_STEP_TICKS, POINTS_PER_CELL,
};
use qix_core::{
    Board, Cell, ClaimThreshold, Dir, DrawKind, Game, GameState, HighScore, HighScores, Initials,
    InputState, Point, Qix, Rng, Session, Sparx, SparxKind, Vec2,
};

use crate::render::high_score::table_rows;
use crate::render::screen::{Screen, game_over, level_complete, paused, screen, title};

/// Seed of every game the tests play.
const SEED: u64 = 1;

/// Seed of [`low_threshold_game`]: its Qix wander off their still start,
/// and under it they keep clear of the stix.
const LEVEL_SEED: u64 = 1206;

/// Dimensions of the board of [`claimed_game`].
const W: u16 = 4;
const H: u16 = 2;

/// The cells the stix of [`claimed_game`] claims.
pub(super) const CLAIMED_CELLS: u32 = 2;

/// Dimensions of the board of [`level_game`]: a stix up column
/// `LEVEL_W / 2` from the marker's start claims the cells right of it,
/// leaving the Qix in cell `(0, 0)` on its left.
const LEVEL_W: u16 = 8;
pub(super) const LEVEL_H: u16 = 3;

/// Cells [`level_game`] claims before the stix, clear of its column and
/// of the top row: with the stix, 18 cells of 24.
const LEVEL_CLAIMED: [(u16, u16); 6] = [(0, 1), (1, 1), (2, 1), (0, 2), (1, 2), (2, 2)];

/// Dimensions of the board of [`split_game`]: a stix up column
/// `SPLIT_W / 2` splits its two Qix.
const SPLIT_W: u16 = 8;
const SPLIT_H: u16 = 4;

/// The columns [`split_game`] claims before the stix: the first walls a
/// Qix into column 0, the others narrow the area of the second Qix.
const SPLIT_CLAIMED_COLUMNS: [u16; 3] = [1, 6, 7];

/// Ticks a game is given to lose every life to a Sparx circling its
/// `W x H` border.
pub(super) const MAX_GAME_TICKS: u32 = 10_000;

fn press(confirm: bool, pause: bool) -> InputState {
    InputState {
        confirm,
        pause,
        ..InputState::default()
    }
}

fn new_session() -> Session {
    Session::new(Game::new(Rng::new(SEED)))
}

/// A game on a `W x H` board with a still Qix in its bottom-right cell
/// that drew a fast stix up and left from the marker's start, claiming
/// the [`CLAIMED_CELLS`] cells of the bottom row's left half.
pub(super) fn claimed_game() -> Game {
    let qix = Qix::new(
        Vec2::new(3.25, 1.5),
        Vec2::new(3.75, 1.5),
        Vec2::default(),
        0.0,
    );
    let mut game = Game::with_board(Board::new(W, H), vec![qix], Vec::new(), Rng::new(SEED));
    for (dir, steps) in [(Dir::Up, 1), (Dir::Left, 2)] {
        let input = InputState {
            dir: Some(dir),
            fast: true,
            ..InputState::default()
        };
        for _ in 0..steps * MARKER_STEP_TICKS {
            game.tick(&input);
        }
    }
    assert_eq!(game.stix(), None, "the stix did not close");
    game
}

/// A still Qix in cell `(x, y)`.
fn qix_at(x: u16, y: u16) -> Qix {
    let (x, y) = (f32::from(x), f32::from(y) + 0.5);
    Qix::new(
        Vec2::new(x + 0.25, y),
        Vec2::new(x + 0.75, y),
        Vec2::default(),
        0.0,
    )
}

/// A game on a `width x height` board at the lowest claim threshold,
/// with `claimed` cells claimed, `qix` and no Sparx.
fn low_threshold_game(
    (width, height): (u16, u16),
    claimed: impl IntoIterator<Item = (u16, u16)>,
    qix: Vec<Qix>,
) -> Game {
    let mut board = Board::new(width, height);
    for (x, y) in claimed {
        board.set_cell(x, y, Cell::Claimed(DrawKind::Fast));
    }
    let mut game = Game::with_board(board, qix, Vec::new(), Rng::new(LEVEL_SEED));
    game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.start()));
    game
}

/// A game whose stix up the middle completes the level at 75%, 25 points
/// above the threshold.
pub(super) fn level_game() -> Game {
    low_threshold_game((LEVEL_W, LEVEL_H), LEVEL_CLAIMED, vec![qix_at(0, 0)])
}

/// A game whose stix up the middle splits a Qix walled into column 0
/// from another in column 5, with 62.5% claimed.
fn split_game() -> Game {
    let claimed = SPLIT_CLAIMED_COLUMNS
        .into_iter()
        .flat_map(|x| (0..SPLIT_H).map(move |y| (x, y)));
    let qix = vec![qix_at(0, 0), qix_at(5, 0)];
    low_threshold_game((SPLIT_W, SPLIT_H), claimed, qix)
}

/// A session over `game`, `height` cells high, started from the title,
/// that drew a fast stix up the middle and completed the level.
pub(super) fn completed_level(game: Game, height: u16) -> Session {
    completed_level_with(game, height, HighScores::default())
}

/// [`completed_level`] in a session with `high_scores`.
pub(super) fn completed_level_with(game: Game, height: u16, high_scores: HighScores) -> Session {
    let mut session = Session::with_high_scores(game, high_scores);
    session.tick(&press(true, false));
    let input = InputState {
        dir: Some(Dir::Up),
        fast: true,
        ..InputState::default()
    };
    for _ in 0..u32::from(height) * MARKER_STEP_TICKS {
        session.tick(&input);
    }
    session
}

/// The screen of `session`, whose state must be a completed level, and
/// the [`level_complete`] screen of that state.
fn level_screens(session: &Session) -> (Option<Screen>, Screen) {
    let GameState::LevelComplete {
        fraction,
        bonus,
        split,
    } = session.state()
    else {
        panic!("level not complete: {:?}", session.state());
    };
    let expected = level_complete(session.game(), fraction, bonus, split);
    (screen(session), expected)
}

/// A session on a free `W x H` board, started from the title, played
/// until its marker lost every life to a Sparx circling the border.
fn lost_game() -> Session {
    let sparx = Sparx {
        pos: Point::new(W / 2 + 1, H),
        dir: Dir::Left,
        kind: SparxKind::Normal,
    };
    let board = Board::new(W, H);
    let game = Game::with_board(
        board,
        vec![qix_at(W - 1, H - 1)],
        vec![sparx],
        Rng::new(SEED),
    );
    let mut session = Session::new(game);
    session.tick(&press(true, false));
    for _ in 0..MAX_GAME_TICKS {
        if session.state() != GameState::Playing {
            break;
        }
        session.tick(&InputState::default());
    }
    session
}

#[test]
fn a_new_session_shows_the_title_screen() {
    assert_eq!(screen(&new_session()), Some(title(&HighScores::default())));
}

#[test]
fn a_session_with_high_scores_shows_them_on_the_title() {
    let table = HighScores::from(vec![
        HighScore {
            name: Initials::try_from("QIX".to_owned()).unwrap(),
            score: 12_345,
        },
        HighScore {
            name: Initials::FIRST,
            score: 10,
        },
    ]);
    let session = Session::with_high_scores(Game::new(Rng::new(SEED)), table.clone());

    let shown = screen(&session).unwrap();

    assert_eq!(shown.table, ["1. QIX  12,345", "2. AAA      10"]);
    assert_eq!(shown, title(&table));
}

#[test]
fn the_title_screen_lists_the_controls() {
    assert_eq!(
        title(&HighScores::default()),
        Screen {
            heading: "QIX".to_owned(),
            table: Vec::new(),
            lines: vec![
                "Up/Down/Left/Right: move".to_owned(),
                "hold Z while moving: draw fast".to_owned(),
                "hold X while moving: draw slow, x2 points".to_owned(),
                "Escape: pause".to_owned(),
                "F1: debug overlay".to_owned(),
                String::new(),
                "Press Enter to start".to_owned(),
            ],
        }
    );
}

#[test]
fn playing_shows_no_screen() {
    let mut session = new_session();
    session.tick(&press(true, false));
    assert_eq!(screen(&session), None);
}

#[test]
fn a_paused_session_shows_the_pause_screen() {
    let mut session = new_session();
    session.tick(&press(true, false));
    session.tick(&press(false, true));
    assert_eq!(screen(&session), Some(paused()));
    assert_eq!(paused().lines, ["Press Escape to resume"]);
}

#[test]
fn a_completed_level_shows_the_claim_the_bonus_and_the_multiplier() {
    let mut game = Game::new(Rng::new(SEED));
    game.set_multiplier(3);
    assert_eq!(
        level_complete(&game, 0.8125, 6000, false),
        Screen {
            heading: "LEVEL 1 COMPLETE".to_owned(),
            table: Vec::new(),
            lines: vec![
                "Claimed 81.2%".to_owned(),
                "Bonus 6,000".to_owned(),
                "Multiplier x3".to_owned(),
                String::new(),
                "Press Enter for level 2".to_owned(),
            ],
        }
    );
}

#[test]
fn a_split_says_so_above_the_claim() {
    let game = Game::new(Rng::new(SEED));
    let lines = level_complete(&game, 0.5, 0, true).lines;
    assert_eq!(lines[..2], ["The Qix are split apart!", "Claimed 50.0%"]);
}

#[test]
fn game_over_shows_the_final_score() {
    let game = claimed_game();
    let points = CLAIMED_CELLS * POINTS_PER_CELL;
    assert_eq!(game.score().points(), points);
    assert_eq!(
        game_over(&game, &HighScores::default()).lines,
        [
            format!("Final score {points}"),
            "A new high score!".to_owned(),
            String::new(),
            "Press Enter to continue".to_owned(),
        ]
    );
}

#[test]
fn game_over_says_nothing_of_a_score_off_the_table() {
    let game = claimed_game();
    let best = HighScore {
        name: Initials::FIRST,
        score: u32::MAX,
    };
    let table = HighScores::from(vec![best; HIGH_SCORE_COUNT]);

    assert_eq!(
        game_over(&game, &table).lines,
        [
            format!("Final score {}", game.score().points()),
            String::new(),
            "Press Enter to continue".to_owned(),
        ]
    );
}

#[test]
fn the_title_screen_lists_the_high_scores_and_the_controls() {
    let table = HighScores::from(vec![HighScore {
        name: Initials::FIRST,
        score: 1,
    }]);

    let title = title(&table);

    assert_eq!(title.table, table_rows(&table));
    assert_eq!(title.lines, self::title(&HighScores::default()).lines);
}

#[test]
fn the_final_score_has_thousands_separators() {
    let session = completed_level(level_game(), LEVEL_H);
    let game = session.game();
    assert_eq!(game.score().points(), 25_120);
    assert_eq!(
        game_over(game, &HighScores::default()).lines[0],
        "Final score 25,120"
    );
}

#[test]
fn a_completed_level_shows_its_claim_and_bonus() {
    let session = completed_level(level_game(), LEVEL_H);
    let (shown, expected) = level_screens(&session);
    assert_eq!(shown, Some(expected));
    let lines = shown.unwrap().lines;
    assert_eq!(lines[..2], ["Claimed 75.0%", "Bonus 25,000"]);
}

#[test]
fn a_split_level_shows_the_split() {
    let session = completed_level(split_game(), SPLIT_H);
    let (shown, expected) = level_screens(&session);
    assert_eq!(shown, Some(expected));
    let lines = shown.unwrap().lines;
    assert_eq!(
        lines[..3],
        ["The Qix are split apart!", "Claimed 62.5%", "Bonus 0"]
    );
}

#[test]
fn a_lost_game_shows_the_game_over_screen() {
    let session = lost_game();
    assert_eq!(session.state(), GameState::GameOver);
    assert_eq!(
        screen(&session),
        Some(game_over(session.game(), session.high_scores()))
    );
}
