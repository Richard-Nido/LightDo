use crate::model::Mode;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Hidden,
    Opening,
    Expanded,
    Closing,
    Pinned,
    Moving,
    Floating,
    Desktop,
}
pub struct WindowState {
    pub phase: Phase,
    pub progress: f32,
    pub target: f32,
    pub mode: Mode,
    pub blocked: bool,
    pub pinned: bool,
    pub away_since: Option<u64>,
    pub edge_since: Option<u64>,
}
impl Default for WindowState {
    fn default() -> Self {
        Self {
            phase: Phase::Expanded,
            progress: 1.0,
            target: 1.0,
            mode: Mode::Edge,
            blocked: false,
            pinned: false,
            away_since: None,
            edge_since: None,
        }
    }
}
impl WindowState {
    pub fn timer_interval(&self, visible: bool, paused: bool, minimized: bool) -> Option<u32> {
        if paused || minimized || (!visible && self.phase != Phase::Hidden) {
            None
        } else if (self.progress - self.target).abs() > 0.001 {
            Some(16)
        } else {
            Some(80)
        }
    }
    pub fn show(&mut self) {
        self.target = 1.0;
        self.phase = match self.mode {
            Mode::Floating => Phase::Floating,
            Mode::Desktop => Phase::Desktop,
            Mode::Edge => {
                if self.pinned {
                    Phase::Pinned
                } else {
                    Phase::Opening
                }
            }
        };
        self.away_since = None;
    }
    pub fn hide(&mut self) {
        if !self.blocked && !self.pinned && self.mode == Mode::Edge {
            self.target = 0.0;
            self.phase = Phase::Closing;
        }
    }
    pub fn mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.progress = 1.0;
        self.target = 1.0;
        self.phase = match mode {
            Mode::Edge => Phase::Expanded,
            Mode::Floating => Phase::Floating,
            Mode::Desktop => Phase::Desktop,
        };
        self.away_since = None;
        self.edge_since = None;
    }
    pub fn step(&mut self, elapsed_ms: f32, reduced: bool) -> bool {
        let previous = self.progress;
        if reduced {
            self.progress = self.target
        } else {
            let step = elapsed_ms / 220.0;
            if self.progress < self.target {
                self.progress = (self.progress + step).min(self.target)
            } else {
                self.progress = (self.progress - step).max(self.target)
            }
        }
        if self.mode == Mode::Edge && self.progress == self.target {
            self.phase = if self.progress == 0.0 {
                Phase::Hidden
            } else if self.pinned {
                Phase::Pinned
            } else {
                Phase::Expanded
            }
        }
        previous != self.progress
    }
    pub fn pointer(
        &mut self,
        now: u64,
        inside: bool,
        at_edge: bool,
        dwell: u64,
        delay: u64,
        auto_hide: bool,
    ) {
        if self.mode != Mode::Edge {
            return;
        }
        if at_edge && self.target == 0.0 {
            let since = *self.edge_since.get_or_insert(now);
            if now.saturating_sub(since) >= dwell {
                self.show();
                self.edge_since = None;
            }
        } else if !at_edge {
            self.edge_since = None;
        }
        if inside || self.blocked || self.pinned || !auto_hide {
            self.away_since = None;
            if self.target == 0.0
                && (self.progress > 0.0 || self.blocked || self.pinned || !auto_hide)
            {
                self.show();
            }
            return;
        }
        if self.target == 1.0 {
            let since = *self.away_since.get_or_insert(now);
            if now.saturating_sub(since) >= delay {
                self.hide();
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reversing_animation_does_not_queue() {
        let mut s = WindowState::default();
        s.hide();
        s.step(110.0, false);
        assert_eq!(s.progress, 0.5);
        s.show();
        s.step(110.0, false);
        assert_eq!(s.progress, 1.0);
        assert_eq!(s.phase, Phase::Expanded);
    }
    #[test]
    fn dwell_and_leave_debounce() {
        let mut s = WindowState::default();
        s.hide();
        s.step(220.0, false);
        s.pointer(0, false, true, 250, 400, true);
        s.pointer(200, false, false, 250, 400, true);
        assert_eq!(s.target, 0.0);
        s.pointer(300, false, true, 250, 400, true);
        s.pointer(550, false, true, 250, 400, true);
        assert_eq!(s.target, 1.0);
        s.pointer(600, true, false, 250, 400, true);
        s.pointer(900, false, false, 250, 400, true);
        s.pointer(1200, true, false, 250, 400, true);
        assert_eq!(s.target, 1.0);
        s.blocked = true;
        s.pointer(5000, false, false, 250, 400, true);
        assert_eq!(s.target, 1.0);
    }
    #[test]
    fn modes_share_state_and_disable_edge_hide() {
        let mut s = WindowState::default();
        for m in [Mode::Floating, Mode::Desktop, Mode::Edge] {
            s.mode(m);
            s.hide();
            assert_eq!(s.target, if m == Mode::Edge { 0.0 } else { 1.0 });
        }
    }
    #[test]
    fn pointer_return_reverses_closing_without_another_dwell() {
        let mut s = WindowState::default();
        s.hide();
        s.step(80.0, false);
        let progress = s.progress;
        s.pointer(100, true, false, 250, 400, true);
        assert_eq!(s.target, 1.0);
        assert_eq!(s.progress, progress);
        s.step(80.0, false);
        assert_eq!(s.progress, 1.0);
    }
    #[test]
    fn tray_and_paused_windows_do_not_poll_but_hidden_edge_still_detects_pointer() {
        let mut s = WindowState::default();
        assert_eq!(s.timer_interval(false, false, false), None);
        assert_eq!(s.timer_interval(true, true, false), None);
        assert_eq!(s.timer_interval(true, false, true), None);
        s.hide();
        assert_eq!(s.timer_interval(true, false, false), Some(16));
        s.step(220.0, false);
        assert_eq!(s.timer_interval(false, false, false), Some(80));
        s.mode(Mode::Floating);
        assert_eq!(s.timer_interval(false, false, false), None);
        assert_eq!(s.timer_interval(true, false, false), Some(80));
    }
}
