pub mod layout;
pub mod widgets;

use std::collections::HashMap;
use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event as CtEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use rusvel_core::domain::{Event, Goal, Opportunity, Task};

use crate::layout::dashboard_layout;
use crate::widgets::{
    events_widget, goals_widget, header_widget, pipeline_stats, pipeline_widget, tasks_widget,
};

/// Data bundle passed into the TUI for display.
#[derive(Debug, Clone)]
pub struct TuiData {
    pub session_name: String,
    pub goals: Vec<Goal>,
    pub tasks: Vec<Task>,
    pub opportunities: Vec<Opportunity>,
    pub recent_events: Vec<Event>,
}

/// Main TUI application state.
pub struct TuiApp {
    session_name: String,
    goals: Vec<Goal>,
    tasks: Vec<Task>,
    pipeline: HashMap<String, usize>,
    recent_events: Vec<Event>,
}

impl TuiApp {
    pub fn new(data: TuiData) -> Self {
        let pipeline = pipeline_stats(&data.opportunities);
        Self {
            session_name: data.session_name,
            goals: data.goals,
            tasks: data.tasks,
            pipeline,
            recent_events: data.recent_events,
        }
    }

    /// Enter the terminal, render the dashboard, and block until the user
    /// presses `q` or `Esc`.
    pub fn run(&self) -> Result<()> {
        enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|frame| {
                let (header_area, grid) = dashboard_layout(frame.area());

                frame.render_widget(header_widget(&self.session_name), header_area);
                frame.render_widget(tasks_widget(&self.tasks), grid[0]);
                frame.render_widget(goals_widget(&self.goals), grid[1]);
                frame.render_widget(pipeline_widget(&self.pipeline), grid[2]);
                frame.render_widget(events_widget(&self.recent_events), grid[3]);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let CtEvent::Key(key) = event::read()? {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        break;
                    }
                }
            }
        }

        disable_raw_mode()?;
        io::stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}

/// Async entry point that surfaces (CLI, API) can call.
pub async fn run_tui(data: TuiData) -> Result<()> {
    let app = TuiApp::new(data);
    // Run the blocking TUI loop on a dedicated thread so we don't block the
    // Tokio runtime.
    tokio::task::spawn_blocking(move || app.run()).await??;
    Ok(())
}
