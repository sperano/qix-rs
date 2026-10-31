//! The fuse: lit when the marker stands still while drawing, it burns
//! along the stix from its start towards the marker.

use crate::tuning::{FUSE_DELAY_TICKS, FUSE_STEP_TICKS};

/// A lit fuse.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fuse {
    /// The index, in [`crate::Stix::points`], of the stix point the fuse
    /// has burnt to.
    pub pos: usize,
}

/// What lights, burns and puts out the fuse of the stix being drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct FuseTimer {
    /// The fuse, `None` while it is out.
    fuse: Option<Fuse>,
    /// Ticks the marker has stood still while the fuse is out, or since
    /// the lit fuse last moved.
    ticks: u32,
}

impl FuseTimer {
    /// The fuse, if it is lit.
    pub(crate) fn fuse(&self) -> Option<Fuse> {
        self.fuse
    }

    /// Puts the fuse out and restarts the delay: the marker stepped, or it
    /// no longer draws a stix.
    pub(crate) fn put_out(&mut self) {
        *self = Self::default();
    }

    /// Advances one tick of a marker standing still at the stix point of
    /// index `head`: a fuse that is out is lit at the stix's start after
    /// [`FUSE_DELAY_TICKS`] such ticks, and a lit one moves one point
    /// towards `head` every [`FUSE_STEP_TICKS`].
    pub(crate) fn stand(&mut self, head: usize) {
        self.ticks += 1;
        match &mut self.fuse {
            None if self.ticks >= FUSE_DELAY_TICKS => self.light(),
            None => {}
            Some(fuse) if self.ticks >= FUSE_STEP_TICKS => {
                self.ticks = 0;
                fuse.pos = (fuse.pos + 1).min(head);
            }
            Some(_) => {}
        }
    }

    /// Lights the fuse at the stix's start at once, unless it is lit
    /// already: the marker is caught in a dead end.
    pub(crate) fn light(&mut self) {
        if self.fuse.is_none() {
            self.fuse = Some(Fuse { pos: 0 });
            self.ticks = 0;
        }
    }

    /// The lit fuse is at the stix point of index `head`, the marker's.
    pub(crate) fn reached(&self, head: usize) -> bool {
        self.fuse.is_some_and(|fuse| fuse.pos == head)
    }
}
