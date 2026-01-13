//! Generic mouse click tracking for double-click detection.
//!
//! Provides reusable functionality to detect double-clicks based on timing and position.
//! Can be used by any panel that needs double-click detection.

use std::time::Instant;
use termide_config::constants::DOUBLE_CLICK_INTERVAL_MS;

/// Generic click tracker with configurable position type.
///
/// The position type `P` allows panels to track clicks in different ways:
/// - `usize` - single index (FileManager)
/// - `(usize, usize)` - line, column (Editor)
/// - Custom section+index types (GitStatusPanel)
#[derive(Debug, Clone, Default)]
pub struct ClickTracker<P = usize> {
    /// Last click time.
    time: Option<Instant>,
    /// Last click position.
    position: Option<P>,
    /// Flag to skip next MouseUp event (useful after double-click selection).
    pub skip_next_up: bool,
}

impl<P: PartialEq + Clone> ClickTracker<P> {
    /// Create a new click tracker.
    pub fn new() -> Self {
        Self {
            time: None,
            position: None,
            skip_next_up: false,
        }
    }

    /// Check if this click is a double-click (same position within threshold).
    pub fn is_double_click(&self, position: &P) -> bool {
        if let (Some(last_time), Some(ref last_pos)) = (self.time, &self.position) {
            let elapsed = Instant::now().duration_since(last_time);
            elapsed.as_millis() < DOUBLE_CLICK_INTERVAL_MS && last_pos == position
        } else {
            false
        }
    }

    /// Check if click at given time and position is a double-click.
    pub fn is_double_click_at(&self, now: Instant, position: &P) -> bool {
        if let (Some(last_time), Some(ref last_pos)) = (self.time, &self.position) {
            let elapsed = now.duration_since(last_time);
            elapsed.as_millis() < DOUBLE_CLICK_INTERVAL_MS && last_pos == position
        } else {
            false
        }
    }

    /// Record a click at the given position (uses current time).
    pub fn record(&mut self, position: P) {
        self.time = Some(Instant::now());
        self.position = Some(position);
    }

    /// Record a click at the given position and time.
    pub fn record_at(&mut self, now: Instant, position: P) {
        self.time = Some(now);
        self.position = Some(position);
    }

    /// Reset click tracking (e.g., after double-click action).
    pub fn reset(&mut self) {
        self.time = None;
        self.position = None;
    }

    /// Get the last recorded position.
    pub fn last_position(&self) -> Option<&P> {
        self.position.as_ref()
    }

    /// Get the last click time.
    pub fn last_time(&self) -> Option<Instant> {
        self.time
    }
}

/// Simple click tracker using single usize index.
pub type IndexClickTracker = ClickTracker<usize>;

/// Click tracker for line/column positions.
pub type PositionClickTracker = ClickTracker<(usize, usize)>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker_no_double_click() {
        let tracker: IndexClickTracker = ClickTracker::new();
        assert!(!tracker.is_double_click(&0));
    }

    #[test]
    fn test_click_sequence() {
        let mut tracker: IndexClickTracker = ClickTracker::new();

        // First click - record it
        tracker.record(5);

        // Checking same position immediately after recording = double-click pattern
        // This is correct: user clicks, we record, then on second click we check
        // is_double_click BEFORE recording the second click
        assert!(tracker.is_double_click(&5));

        // After reset, no double-click
        tracker.reset();
        assert!(!tracker.is_double_click(&5));
    }

    #[test]
    fn test_no_double_click_different_position() {
        let mut tracker: IndexClickTracker = ClickTracker::new();
        tracker.record(5);
        // Different position should not be a double-click
        assert!(!tracker.is_double_click(&10));
    }

    #[test]
    fn test_reset() {
        let mut tracker: IndexClickTracker = ClickTracker::new();
        tracker.record(5);
        tracker.reset();
        assert!(!tracker.is_double_click(&5));
        assert!(tracker.last_position().is_none());
    }

    #[test]
    fn test_position_tracker() {
        let mut tracker: PositionClickTracker = ClickTracker::new();
        tracker.record((10, 5));
        assert!(tracker.is_double_click(&(10, 5)));
        assert!(!tracker.is_double_click(&(10, 6)));
        assert!(!tracker.is_double_click(&(11, 5)));
    }
}
