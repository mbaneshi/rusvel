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
/// │   (bar chart)  │    (timeline)   │
/// └────────────────┴─────────────────┘
/// ```
pub fn dashboard_layout(area: Rect) -> (Rect, [Rect; 4]) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(8),    // top row
            Constraint::Min(8),    // bottom row
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

    (rows[0], [top[0], top[1], bottom[0], bottom[1]])
}
