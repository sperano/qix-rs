//! The rules screen's controls: the pages of the game's rules, one shown
//! at a time.
//!
//! Left and right turn the page, stopping at the first and the last;
//! confirm turns to the next page, or leaves from the last one, and
//! overrides a direction in the same tick; pause leaves. Directions
//! repeat as [`DirRepeat`] reads them.

use qix_core::{Dir, DirRepeat, InputState};

/// A page of the rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    /// The goal, and how the marker draws and claims.
    HowToPlay,
    /// The Qix, the Sparx and the fuse.
    Dangers,
    /// The points, the bonus, the multiplier and the lives.
    Scoring,
}

/// The pages of the rules, in reading order.
pub const PAGES: [Page; 3] = [Page::HowToPlay, Page::Dangers, Page::Scoring];

/// The rules screen while it is open: the page shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RulesBook {
    /// The index in [`PAGES`] of the page shown.
    page: usize,
    repeat: DirRepeat,
}

impl RulesBook {
    /// The book open at its first page.
    pub fn new() -> Self {
        Self::default()
    }

    /// The page shown.
    pub fn page(&self) -> Page {
        PAGES[self.page]
    }

    /// The index of the page shown in [`PAGES`], from zero.
    pub fn page_index(&self) -> usize {
        self.page
    }

    /// Whether the page shown is the last one.
    pub fn is_last_page(&self) -> bool {
        self.page + 1 == PAGES.len()
    }

    /// Applies one tick of `input`; returns whether the screen closes.
    pub fn tick(&mut self, input: &InputState) -> bool {
        if input.pause {
            return true;
        }
        let dir = self.repeat.tick(input.dir);
        if input.confirm {
            return self.confirm();
        }
        match dir {
            Some(Dir::Left) => self.page = self.page.saturating_sub(1),
            Some(Dir::Right) => self.next_page(),
            Some(Dir::Up | Dir::Down) | None => {}
        }
        false
    }

    /// Turns to the next page; returns whether the screen closes, from
    /// the last page.
    fn confirm(&mut self) -> bool {
        if self.is_last_page() {
            return true;
        }
        self.next_page();
        false
    }

    /// Turns to the next page; stays on the last one.
    fn next_page(&mut self) {
        self.page = (self.page + 1).min(PAGES.len() - 1);
    }
}
