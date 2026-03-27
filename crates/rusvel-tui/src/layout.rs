use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Compute the dashboard layout:
/// ```text
/// ┌──────────────────────────────────┐
/// │         RUSVEL - Session X        │
/// ├────────────────┬─────────────────┤
/// │   Today's      │    Goals        │
/// │   Tasks        │    (table)      │
/// ├────────────────┼─────────────────┤
/// │   Pipeline     │    Events       │
/// ├────────────────┴─────────────────┤
/// │   Terminal (panes | output)       │
/// └──────────────────────────────────┘
/// ```
pub fn dashboard_layout(area: Rect) -> (Rect, [Rect; 4], Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Min(6),
            Constraint::Min(7),
        ])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[2]);

    (rows[0], [top[0], top[1], bottom[0], bottom[1]], rows[3])
}

/// Split the terminal strip: pane list | output.
pub fn terminal_split(area: Rect) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(area);
    [chunks[0], chunks[1]]
}
