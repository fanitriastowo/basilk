use ratatui::widgets::ListState;

/// Single-list selection + scroll state.
///
/// Wraps `ListState` so every list in `App` shares one implementation of
/// wrap-around navigation, clamping after mutations, and delete handling
/// instead of scattering raw `select(...)` / `select_previous()` calls.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    state: ListState,
}

impl Selection {
    pub fn with_selected(index: Option<usize>) -> Self {
        Self {
            state: ListState::default().with_selected(index),
        }
    }

    /// Default selection: first row (also used for empty lists, matching the
    /// historic `Some(0)` convention so guards on `items.is_empty()` keep
    /// working).
    pub fn first() -> Self {
        Self::with_selected(Some(0))
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    /// Mutable `ListState` for `render_stateful_widget`.
    pub fn state(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn select_first(&mut self) {
        self.select(Some(0));
    }

    pub fn select_last(&mut self, len: usize) {
        if len == 0 {
            self.select(Some(0));
        } else {
            self.select(Some(len - 1));
        }
    }

    /// Clamp the selection into `0..len` (empty lists keep `Some(0)`).
    pub fn clamp(&mut self, len: usize) {
        if len == 0 {
            self.select(Some(0));
            return;
        }
        let clamped = self.selected().unwrap_or(0).min(len - 1);
        self.select(Some(clamped));
    }

    /// Wrap-around step used by `j/k`, arrows and modal navigation.
    pub fn move_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let cur = self.selected().unwrap_or(0).min(len - 1);
        self.select(Some((cur + 1) % len));
    }

    pub fn move_prev(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let cur = self.selected().unwrap_or(0).min(len - 1);
        self.select(Some(if cur == 0 { len - 1 } else { cur - 1 }));
    }

    /// Selection after deleting row `deleted` from a list that now has `len`
    /// rows: the predecessor (or the successor when the first row was
    /// deleted, or the new last row when the old last row was deleted).
    pub fn after_delete(&mut self, deleted: usize, len: usize) {
        if len == 0 {
            self.select(Some(0));
        } else if deleted == 0 {
            self.select(Some(0));
        } else {
            self.select(Some((deleted - 1).min(len - 1)));
        }
    }
}

/// Kanban board focus: the focused lane plus one remembered row per lane.
///
/// The full-list `selected_task_index` remains the action target; this only
/// tracks where the highlight lives per lane so switching lanes restores the
/// previous row.
#[derive(Debug, Clone)]
pub struct Board {
    pub lane: usize,
    pub rows: [Selection; 5],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            lane: 0,
            rows: [
                Selection::first(),
                Selection::first(),
                Selection::first(),
                Selection::first(),
                Selection::first(),
            ],
        }
    }

    pub fn row(&self, lane: usize) -> Option<usize> {
        self.rows[lane].selected()
    }

    pub fn select_row(&mut self, lane: usize, row: usize) {
        self.rows[lane].select(Some(row));
    }

    pub fn clamp_row(&mut self, lane: usize, len: usize) {
        self.rows[lane].clamp(len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_next_wraps_and_ignores_empty() {
        let mut s = Selection::first();
        s.move_next(0);
        assert_eq!(s.selected(), Some(0));

        let mut s = Selection::with_selected(Some(1));
        s.move_next(2);
        assert_eq!(s.selected(), Some(0));
        s.move_next(2);
        assert_eq!(s.selected(), Some(1));
    }

    #[test]
    fn move_prev_wraps_and_clamps_oob() {
        let mut s = Selection::first();
        s.move_prev(2);
        assert_eq!(s.selected(), Some(1));

        let mut s = Selection::with_selected(Some(9));
        s.move_prev(2);
        assert_eq!(s.selected(), Some(0));
    }

    #[test]
    fn clamp_keeps_selection_in_bounds() {
        let mut s = Selection::with_selected(Some(5));
        s.clamp(2);
        assert_eq!(s.selected(), Some(1));

        let mut s = Selection::with_selected(Some(0));
        s.clamp(0);
        assert_eq!(s.selected(), Some(0));
    }

    #[test]
    fn after_delete_selects_predecessor() {
        let mut s = Selection::first();
        s.after_delete(1, 2);
        assert_eq!(s.selected(), Some(0));

        let mut s = Selection::first();
        s.after_delete(2, 2);
        assert_eq!(s.selected(), Some(1));

        let mut s = Selection::first();
        s.after_delete(0, 0);
        assert_eq!(s.selected(), Some(0));
    }
}
