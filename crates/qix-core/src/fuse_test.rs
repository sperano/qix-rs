//! The fuse lighting, burning and going out through [`Game::tick`].

use crate::fuse::FuseTimer;
use crate::stix::legal_moves;
use crate::tuning::{FUSE_DELAY_TICKS, FUSE_STEP_TICKS, MARKER_STEP_TICKS};
use crate::{
    Board, DeathCause, Dir, Event, Fuse, Game, InputState, MarkerState, Point, Qix, Rng, Vec2,
};

/// Seed of every game the tests play.
const SEED: u64 = 1196;

/// Dimensions of the boards: the marker starts at `(W / 2, H)`.
const W: u16 = 8;
const H: u16 = 5;

/// Index of the head of a stix drawn up, right, then up.
const UP_RIGHT_UP_HEAD: usize = 3;

/// Index of the head of [`spiral_into_a_dead_end`]'s stix.
const SPIRAL_HEAD: usize = 5;

/// A game on a free `W x H` board with a short Qix in its top-left cell
/// and no Sparx.
fn fuse_game() -> Game {
    let qix = Qix::new(
        Vec2::new(0.25, 0.5),
        Vec2::new(0.75, 0.5),
        Vec2::default(),
        0.0,
    );
    Game::with_board(Board::new(W, H), qix, Vec::new(), Rng::new(SEED))
}

fn drawing(dir: Dir) -> InputState {
    InputState {
        dir: Some(dir),
        fast: true,
        ..InputState::default()
    }
}

/// Ticks `game` `ticks` times with the same input.
fn hold(game: &mut Game, input: InputState, ticks: u32) {
    for _ in 0..ticks {
        game.tick(&input);
    }
}

/// Ticks `game` long enough for `steps` fast stix steps towards `dir`.
fn draw(game: &mut Game, dir: Dir, steps: u32) {
    hold(game, drawing(dir), steps * MARKER_STEP_TICKS);
}

/// Ticks `game` `ticks` times with nothing held.
fn stand(game: &mut Game, ticks: u32) {
    hold(game, InputState::default(), ticks);
}

/// A game drawing a stix up 2 steps, right 2, then up 1.
fn up_right_up() -> Game {
    let mut game = fuse_game();
    draw(&mut game, Dir::Up, 2);
    draw(&mut game, Dir::Right, 2);
    draw(&mut game, Dir::Up, 1);
    game
}

/// Draws a stix that spirals in until the marker, at `(5, 2)`, is boxed
/// in by it on every side.
fn spiral_into_a_dead_end(game: &mut Game) {
    draw(game, Dir::Up, 4);
    draw(game, Dir::Right, 2);
    draw(game, Dir::Down, 2);
    draw(game, Dir::Left, 1);
    draw(game, Dir::Up, 1);
}

fn fuse_at(pos: usize) -> Option<Fuse> {
    Some(Fuse { pos })
}

fn fuse_death(game: &Game) -> bool {
    game.events().contains(&Event::Death(DeathCause::Fuse))
}

#[test]
fn the_fuse_is_out_while_the_marker_draws_steadily() {
    let mut game = fuse_game();
    for _ in 0..H - 1 {
        draw(&mut game, Dir::Up, 1);
        assert_eq!(game.fuse(), None);
    }
}

#[test]
fn the_fuse_lights_at_the_stix_start_after_the_delay() {
    let mut game = fuse_game();
    draw(&mut game, Dir::Up, 2);
    stand(&mut game, FUSE_DELAY_TICKS - 1);
    assert_eq!(game.fuse(), None);
    stand(&mut game, 1);
    assert_eq!(game.fuse(), fuse_at(0));
}

#[test]
fn a_marker_on_the_border_never_lights_the_fuse() {
    let mut game = fuse_game();
    stand(&mut game, FUSE_DELAY_TICKS * 2);
    assert_eq!(game.fuse(), None);
}

#[test]
fn the_lit_fuse_burns_one_segment_every_step_ticks() {
    let mut game = up_right_up();
    assert_eq!(game.stix().unwrap().points().len(), UP_RIGHT_UP_HEAD + 1);
    stand(&mut game, FUSE_DELAY_TICKS);
    for pos in 1..=UP_RIGHT_UP_HEAD {
        stand(&mut game, FUSE_STEP_TICKS - 1);
        assert_eq!(game.fuse(), fuse_at(pos - 1));
        assert!(!fuse_death(&game));
        stand(&mut game, 1);
        assert_eq!(game.fuse(), fuse_at(pos));
    }
}

#[test]
fn the_fuse_reaching_the_marker_kills_on_every_tick_it_is_there() {
    let mut game = up_right_up();
    let burn = FUSE_DELAY_TICKS + FUSE_STEP_TICKS * UP_RIGHT_UP_HEAD as u32;
    let deaths: Vec<u32> = (1..=burn + 1)
        .filter(|_| {
            game.tick(&InputState::default());
            fuse_death(&game)
        })
        .collect();
    assert_eq!(deaths, [burn, burn + 1]);
    assert_eq!(game.fuse(), fuse_at(UP_RIGHT_UP_HEAD));
}

#[test]
fn standing_against_an_illegal_step_lights_the_fuse() {
    let mut game = fuse_game();
    draw(&mut game, Dir::Up, 2);
    let pos = game.marker().pos;
    hold(&mut game, drawing(Dir::Down), FUSE_DELAY_TICKS);
    assert_eq!(game.marker().pos, pos);
    assert_eq!(game.fuse(), fuse_at(0));
}

#[test]
fn a_step_puts_the_fuse_out_and_the_next_one_lights_at_the_start() {
    let mut game = up_right_up();
    stand(&mut game, FUSE_DELAY_TICKS + FUSE_STEP_TICKS);
    assert_eq!(game.fuse(), fuse_at(1));
    draw(&mut game, Dir::Up, 1);
    assert_eq!(game.fuse(), None);
    stand(&mut game, FUSE_DELAY_TICKS - 1);
    assert_eq!(game.fuse(), None);
    stand(&mut game, 1);
    assert_eq!(game.fuse(), fuse_at(0));
}

#[test]
fn closing_the_stix_puts_the_fuse_out() {
    let mut game = fuse_game();
    draw(&mut game, Dir::Up, 2);
    stand(&mut game, FUSE_DELAY_TICKS);
    draw(&mut game, Dir::Up, u32::from(H) - 2);
    assert_eq!(game.marker().state, MarkerState::OnBorder);
    assert_eq!(game.fuse(), None);
    stand(&mut game, FUSE_DELAY_TICKS);
    assert_eq!(game.fuse(), None);
}

#[test]
fn a_dead_end_lights_the_fuse_at_once() {
    let mut game = fuse_game();
    spiral_into_a_dead_end(&mut game);
    assert_eq!(game.marker().pos, Point::new(5, 2));
    let stix = game.stix().unwrap();
    assert!(legal_moves(&game.marker(), stix, game.board()).is_empty());
    assert_eq!(stix.points().len(), SPIRAL_HEAD + 1);
    assert_eq!(game.fuse(), fuse_at(0));
}

#[test]
fn the_fuse_is_out_one_step_before_the_dead_end() {
    let mut game = fuse_game();
    draw(&mut game, Dir::Up, 4);
    draw(&mut game, Dir::Right, 2);
    draw(&mut game, Dir::Down, 2);
    draw(&mut game, Dir::Left, 1);
    assert_eq!(game.fuse(), None);
}

#[test]
fn the_fuse_lit_by_a_dead_end_burns_to_the_marker() {
    let mut game = fuse_game();
    spiral_into_a_dead_end(&mut game);
    let burn = FUSE_STEP_TICKS * SPIRAL_HEAD as u32;
    stand(&mut game, burn - 1);
    assert!(!fuse_death(&game));
    stand(&mut game, 1);
    assert!(fuse_death(&game));
}

#[test]
fn lighting_a_lit_fuse_leaves_it_where_it_burnt_to() {
    let mut timer = FuseTimer::default();
    timer.light();
    for _ in 0..FUSE_STEP_TICKS {
        timer.stand(UP_RIGHT_UP_HEAD);
    }
    timer.light();
    assert_eq!(timer.fuse(), fuse_at(1));
}
