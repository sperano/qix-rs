//! Board invariants over random legal draw sequences played through
//! [`Game::tick`]: plain loops over the game's seeded generator, until the
//! simulation harness brings `proptest`.

use std::cell::Cell;

use qix_core::stix::legal_moves;
use qix_core::tuning::{CLAIM_THRESHOLD_RANGE, SLOW_STEP_TICKS};
use qix_core::{
    Board, ClaimThreshold, Dir, DrawKind, Event, Game, InputState, MarkerState, Qix, Sparx,
    SparxKind, Stix, Vec2,
};

/// Dimensions of the board the runs play on: small, so that random stix
/// reach the border before they wind themselves into a dead end.
const W: u16 = 12;
const H: u16 = 9;

/// Seeds of the runs, one deterministic run each.
const SEEDS: [u64; 8] = [1, 2, 3, 5, 8, 13, 21, 34];

/// Games a run plays, each on a fresh board.
const GAMES_PER_RUN: u32 = 20;

/// Moves a game plays at most; a stix caught in a dead end ends it sooner.
const MOVES_PER_GAME: u32 = 300;

/// Chance, in percent, that the marker leaves the border when it can.
const START_DRAW_PERCENT: u64 = 30;

/// Chance, in percent, that a stix keeps its heading when it can: straight
/// runs reach the border more often than a random walk.
const KEEP_HEADING_PERCENT: u64 = 70;

/// Chance, in percent, that a stix step is drawn fast rather than slow.
const FAST_DRAW_PERCENT: u64 = 50;

/// Stix a run must close for the invariants to have been put to the test.
const MIN_CLOSES_PER_RUN: u32 = 20;

/// Seed of the generator each game's Qix draws from.
const QIX_SEED: u64 = 1194;

/// The whole of a [`Rng::chance`].
const PERCENT: u64 = 100;

/// The game's seeded generator, with the picks the runs make.
struct Rng(qix_core::Rng);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// True with a chance of `percent` in a hundred.
    fn chance(&mut self, percent: u64) -> bool {
        self.next_u64() % PERCENT < percent
    }

    /// One of `items` at random, or `None` when there is none.
    fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        let len = u64::try_from(items.len()).ok().filter(|&len| len > 0)?;
        let index = usize::try_from(self.next_u64() % len).ok()?;
        items.get(index).copied()
    }

    fn draw_kind(&mut self) -> DrawKind {
        if self.chance(FAST_DRAW_PERCENT) {
            DrawKind::Fast
        } else {
            DrawKind::Slow
        }
    }
}

/// One marker step: the direction held, with the draw button, if any.
#[derive(Clone, Copy, Debug)]
struct Move {
    dir: Dir,
    draw: Option<DrawKind>,
}

impl Move {
    fn input(self) -> InputState {
        InputState {
            dir: Some(self.dir),
            fast: self.draw == Some(DrawKind::Fast),
            slow: self.draw == Some(DrawKind::Slow),
            ..InputState::default()
        }
    }
}

/// The game just before a tick, as the invariants compare it.
struct Before {
    claimed: usize,
}

impl Before {
    fn of(game: &Game) -> Self {
        Self {
            claimed: claimed_cells(game.board()),
        }
    }
}

/// The number of claimed cells on `board`.
fn claimed_cells(board: &Board) -> usize {
    let cells = (0..board.height()).flat_map(|y| (0..board.width()).map(move |x| (x, y)));
    cells
        .filter(|&(x, y)| board.cell(i32::from(x), i32::from(y)).is_claimed())
        .count()
}

/// A random legal move for `game`'s marker, or `None` when its stix is
/// caught in a dead end.
fn random_move(game: &Game, rng: &mut Rng) -> Option<Move> {
    match game.stix() {
        Some(stix) => stix_move(game, stix, rng),
        None => Some(border_move(game, rng)),
    }
}

/// From the border: a step off it into free space with a draw button
/// held, or else a step along a border edge; only the latter during the
/// grace period after a death, when the marker cannot draw.
fn border_move(game: &Game, rng: &mut Rng) -> Move {
    let (marker, board) = (game.marker(), game.board());
    let starts = match game.grace_ticks() {
        0 => legal_moves(&marker, &Stix::new(marker.pos), board),
        _ => Vec::new(),
    };
    if let Some(dir) = rng.pick(&starts).filter(|_| rng.chance(START_DRAW_PERCENT)) {
        let draw = Some(rng.draw_kind());
        return Move { dir, draw };
    }
    let walks: Vec<Dir> = board
        .border_neighbours(marker.pos)
        .map(|(dir, _)| dir)
        .collect();
    let dir = rng.pick(&walks).expect("a border point has border edges");
    Move { dir, draw: None }
}

/// The next step of `stix`: its heading most of the time when that is
/// legal, otherwise any legal direction.
fn stix_move(game: &Game, stix: &Stix, rng: &mut Rng) -> Option<Move> {
    let moves = legal_moves(&game.marker(), stix, game.board());
    let heading = match stix.points() {
        [.., before, head] => Dir::between(*before, *head),
        _ => None,
    };
    let dir = heading
        .filter(|dir| moves.contains(dir) && rng.chance(KEEP_HEADING_PERCENT))
        .or_else(|| rng.pick(&moves))?;
    let draw = Some(rng.draw_kind());
    Some(Move { dir, draw })
}

/// Holds `step` until the marker moves or dies, calling `check` after
/// every tick, and returns whether the step closed a stix: a draw step
/// that ends on the border, which also holds for a stix closed by its
/// first step.
fn play(game: &mut Game, step: Move, check: &impl Fn(&Before, &Game)) -> bool {
    let from = game.marker().pos;
    let input = step.input();
    for _ in 0..SLOW_STEP_TICKS {
        let before = Before::of(game);
        game.tick(&input);
        check(&before, game);
        if game
            .events()
            .iter()
            .any(|event| matches!(event, Event::Death(_)))
        {
            return false;
        }
        if game.marker().pos != from {
            return step.draw.is_some() && game.marker().state == MarkerState::OnBorder;
        }
    }
    panic!("legal move {step:?} from {from:?} did not step");
}

/// A game on a fresh `W x H` board, with a Qix one unit long at its
/// centre, the two Sparx of a level start, the second of them Super so
/// that it chases along the stix, and the highest claim threshold, so the
/// runs claim as much as they can before the level is complete.
fn new_game() -> Game {
    let board = Board::new(W, H);
    let centre = Vec2::new(f32::from(W), f32::from(H)) * 0.5;
    let half = Vec2::new(0.5, 0.0);
    let qix = Qix::new(centre - half, centre + half, Vec2::default(), 0.0);
    let [normal, chaser] = Sparx::spawn_pair(&board);
    let chaser = Sparx {
        kind: SparxKind::Super,
        ..chaser
    };
    let sparx = vec![normal, chaser];
    let mut game = Game::with_board(board, qix, sparx, qix_core::Rng::new(QIX_SEED));
    game.set_claim_threshold(ClaimThreshold::new(*CLAIM_THRESHOLD_RANGE.end()));
    game
}

/// Plays one game of random legal moves, until it is over or its level
/// complete at the latest, and returns the stix it closed.
fn play_game(rng: &mut Rng, check: &impl Fn(&Before, &Game)) -> u32 {
    let mut game = new_game();
    let mut closes = 0;
    for _ in 0..MOVES_PER_GAME {
        if game.is_over() || game.is_level_complete() {
            break;
        }
        let Some(step) = random_move(&game, rng) else {
            break;
        };
        closes += u32::from(play(&mut game, step, check));
    }
    closes
}

/// Plays every seeded run, calling `check` after every tick, and makes
/// sure each run closed enough stix to have tested something.
fn for_every_tick(check: impl Fn(&Before, &Game)) {
    for seed in SEEDS {
        let mut rng = Rng(qix_core::Rng::new(seed));
        let closes: u32 = (0..GAMES_PER_RUN)
            .map(|_| play_game(&mut rng, &check))
            .sum();
        assert!(
            closes >= MIN_CLOSES_PER_RUN,
            "seed {seed} closed only {closes} stix"
        );
    }
}

#[test]
fn claimed_cell_count_never_decreases() {
    for_every_tick(|before, game| {
        let claimed = claimed_cells(game.board());
        assert!(claimed >= before.claimed, "{} -> {claimed}", before.claimed);
    });
}

#[test]
fn a_stix_exists_exactly_while_drawing() {
    // Covers every close, a stix closed by its first step included: the
    // marker is back on the border, so no stix may remain.
    for_every_tick(|_, game| {
        let drawing = game.marker().state == MarkerState::Drawing;
        assert_eq!(game.stix().is_some(), drawing, "{:?}", game.marker());
    });
}

#[test]
fn marker_is_on_the_border_when_not_drawing() {
    for_every_tick(|_, game| {
        let marker = game.marker();
        if marker.state == MarkerState::OnBorder {
            assert!(game.board().is_border(marker.pos), "{marker:?}");
        }
    });
}

#[test]
fn every_sparx_is_on_the_border_or_chasing_along_the_stix() {
    let off_the_border = Cell::new(0_u32);
    for_every_tick(|_, game| {
        for sparx in game.sparx() {
            let on_border = game.board().is_border(sparx.pos);
            let chasing = game.stix().is_some_and(|stix| sparx.chases_along(stix));
            assert!(on_border || chasing, "{sparx:?}");
            off_the_border.set(off_the_border.get() + u32::from(!on_border));
        }
    });
    assert!(
        off_the_border.get() > 0,
        "no Sparx ever chased along a stix"
    );
}

#[test]
fn claimed_fraction_is_the_claimed_share_of_the_cells() {
    for_every_tick(|_, game| {
        let board = game.board();
        let cells = usize::from(board.width()) * usize::from(board.height());
        // Both sides round the same quotient of two small integers, so
        // they are equal to the bit.
        let expected = claimed_cells(board) as f32 / cells as f32;
        assert_eq!(board.claimed_fraction(), expected);
    });
}
