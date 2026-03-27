//! Interactive REPL shell — Tier 2.
//!
//! Launch with `rusvel shell`. Provides a persistent prompt with
//! autocomplete, history, department context switching, and all
//! the same commands as Tier 1 one-shot CLI.

use std::sync::Arc;

use reedline::{
    ColumnarMenu, Completer, DefaultHinter, EditCommand, Emacs, FileBackedHistory, KeyCode,
    KeyModifiers, MenuBuilder, Prompt, PromptEditMode, PromptHistorySearch,
    PromptHistorySearchStatus, Reedline, ReedlineEvent, ReedlineMenu, Signal, Span, Suggestion,
    default_emacs_keybindings,
};
use rusvel_core::error::Result;
use rusvel_core::ports::{SessionPort, StoragePort};

use crate::departments;

// ── Prompt ────────────────────────────────────────────────────────

struct RusvelPrompt {
    department: Option<String>,
}

impl Prompt for RusvelPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<'_, str> {
        match &self.department {
            Some(dept) => format!("rusvel:{dept}").into(),
            None => "rusvel".into(),
        }
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<'_, str> {
        "".into()
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> std::borrow::Cow<'_, str> {
        "> ".into()
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<'_, str> {
        ".. ".into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> std::borrow::Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "(failed) ",
        };
        format!("{prefix}search: ").into()
    }
}

// ── Completer ────────────────────────────────────────────────────

#[derive(Clone)]
struct RusvelCompleter {
    department: Option<String>,
}

impl RusvelCompleter {
    fn top_level_commands() -> Vec<&'static str> {
        vec![
            "help",
            "exit",
            "quit",
            "use",
            "status",
            "list",
            "events",
            "session",
            "forge",
            "dashboard",
            // departments
            "finance",
            "growth",
            "distro",
            "legal",
            "support",
            "infra",
            "product",
            "code",
            "harvest",
            "content",
            "gtm",
        ]
    }

    fn dept_subcommands() -> Vec<&'static str> {
        vec!["list", "status", "events", "back"]
    }
}

impl Completer for RusvelCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let input = &line[..pos];
        let words: Vec<&str> = input.split_whitespace().collect();

        let candidates: Vec<&str> = if self.department.is_some() {
            // Inside a department context
            Self::dept_subcommands()
        } else if words.len() <= 1 {
            Self::top_level_commands()
        } else {
            // Second word — subcommands for known commands
            match words[0] {
                "session" => vec!["create", "list", "switch"],
                "forge" => vec!["mission"],
                "use" => departments::department_names().to_vec(),
                d if departments::department_names().contains(&d) => Self::dept_subcommands(),
                _ => vec![],
            }
        };

        let partial = words.last().copied().unwrap_or("");
        let start = if partial.is_empty() {
            pos
        } else {
            pos - partial.len()
        };

        candidates
            .into_iter()
            .filter(|c| c.starts_with(partial))
            .map(|c| Suggestion {
                value: c.to_string(),
                description: None,
                style: None,
                extra: None,
                span: Span::new(start, pos),
                append_whitespace: true,
            })
            .collect()
    }
}

// ── Shell runner ─────────────────────────────────────────────────

pub struct ShellContext {
    pub sessions: Arc<dyn SessionPort>,
    pub storage: Arc<dyn StoragePort>,
    pub forge: Arc<forge_engine::ForgeEngine>,
}

pub async fn run_shell(ctx: ShellContext) -> Result<()> {
    println!("RUSVEL Interactive Shell");
    println!("Type 'help' for commands, 'exit' to quit.\n");

    let mut department: Option<String> = None;

    // Build reedline with completions, history, hints
    let history_path = crate::rusvel_dir().join("shell_history.txt");
    let history = Box::new(
        FileBackedHistory::with_file(500, history_path).expect("Failed to create shell history"),
    );

    let completer = Box::new(RusvelCompleter {
        department: department.clone(),
    });
    let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));

    let hinter = Box::new(DefaultHinter::default());

    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('d'),
        ReedlineEvent::Edit(vec![EditCommand::Clear]),
    );
    let edit_mode = Box::new(Emacs::new(keybindings));

    let mut line_editor = Reedline::create()
        .with_history(history)
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_hinter(hinter)
        .with_edit_mode(edit_mode);

    loop {
        let prompt = RusvelPrompt {
            department: department.clone(),
        };

        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let words: Vec<&str> = line.split_whitespace().collect();

                // Handle department context
                if let Some(ref dept) = department {
                    match words[0] {
                        "back" | ".." => {
                            println!("Left {dept} department.");
                            department = None;
                            // Update completer
                            let c = Box::new(RusvelCompleter { department: None });
                            line_editor = line_editor.with_completer(c);
                            continue;
                        }
                        "list" => {
                            let kind = words.get(1).map(std::string::ToString::to_string);
                            let limit = 20;
                            let action = crate::departments::DeptAction::List { kind, limit };
                            let cmd = dept_to_cmd(dept, action);
                            if let Some(cmd) = cmd {
                                let _ = departments::handle_dept(
                                    cmd,
                                    ctx.storage.clone(),
                                    &departments::EngineRefs {
                                        code: None,
                                        content: None,
                                        harvest: None,
                                    },
                                )
                                .await;
                            }
                            continue;
                        }
                        "status" => {
                            let action = crate::departments::DeptAction::Status;
                            let cmd = dept_to_cmd(dept, action);
                            if let Some(cmd) = cmd {
                                let _ = departments::handle_dept(
                                    cmd,
                                    ctx.storage.clone(),
                                    &departments::EngineRefs {
                                        code: None,
                                        content: None,
                                        harvest: None,
                                    },
                                )
                                .await;
                            }
                            continue;
                        }
                        "events" => {
                            let action = crate::departments::DeptAction::Events { limit: 10 };
                            let cmd = dept_to_cmd(dept, action);
                            if let Some(cmd) = cmd {
                                let _ = departments::handle_dept(
                                    cmd,
                                    ctx.storage.clone(),
                                    &departments::EngineRefs {
                                        code: None,
                                        content: None,
                                        harvest: None,
                                    },
                                )
                                .await;
                            }
                            continue;
                        }
                        _ => {
                            println!("Unknown command in {} context: {}", dept, words[0]);
                            println!("Available: list, status, events, back");
                            continue;
                        }
                    }
                }

                // Top-level commands
                match words[0] {
                    "exit" | "quit" | "q" => {
                        println!("Goodbye.");
                        break;
                    }
                    "help" | "?" => {
                        print_help();
                    }
                    "use" => {
                        if let Some(&dept_name) = words.get(1) {
                            if departments::department_names().contains(&dept_name) {
                                println!("Switched to {dept_name} department.");
                                department = Some(dept_name.to_string());
                                let c = Box::new(RusvelCompleter {
                                    department: department.clone(),
                                });
                                line_editor = line_editor.with_completer(c);
                            } else {
                                println!("Unknown department: {dept_name}");
                                println!(
                                    "Available: {}",
                                    departments::department_names().join(", ")
                                );
                            }
                        } else {
                            println!("Usage: use <department>");
                            println!("Available: {}", departments::department_names().join(", "));
                        }
                    }
                    "status" => {
                        // Show all departments summary
                        for &dept in departments::department_names() {
                            let action = crate::departments::DeptAction::Status;
                            if let Some(cmd) = dept_to_cmd(dept, action) {
                                let _ = departments::handle_dept(
                                    cmd,
                                    ctx.storage.clone(),
                                    &departments::EngineRefs {
                                        code: None,
                                        content: None,
                                        harvest: None,
                                    },
                                )
                                .await;
                            }
                        }
                    }
                    "session" => match words.get(1).copied() {
                        Some("list") => {
                            let cmd = crate::SessionCmd::List;
                            let _ = crate::handle_session(cmd, ctx.sessions.clone()).await;
                        }
                        Some("create") => {
                            if let Some(&name) = words.get(2) {
                                let cmd = crate::SessionCmd::Create {
                                    name: name.to_string(),
                                };
                                let _ = crate::handle_session(cmd, ctx.sessions.clone()).await;
                            } else {
                                println!("Usage: session create <name>");
                            }
                        }
                        Some("switch") => {
                            if let Some(&id) = words.get(2) {
                                let cmd = crate::SessionCmd::Switch { id: id.to_string() };
                                let _ = crate::handle_session(cmd, ctx.sessions.clone()).await;
                            } else {
                                println!("Usage: session switch <id>");
                            }
                        }
                        _ => {
                            println!("Usage: session [list|create <name>|switch <id>]");
                        }
                    },
                    "dashboard" => {
                        println!("Launch the TUI dashboard with: rusvel --tui");
                    }
                    // Direct department access without `use`
                    dept if departments::department_names().contains(&dept) => {
                        let sub = words.get(1).copied().unwrap_or("status");
                        let action = match sub {
                            "list" => {
                                let kind = words.get(2).map(std::string::ToString::to_string);
                                crate::departments::DeptAction::List { kind, limit: 20 }
                            }
                            "events" => crate::departments::DeptAction::Events { limit: 10 },
                            _ => crate::departments::DeptAction::Status,
                        };
                        if let Some(cmd) = dept_to_cmd(dept, action) {
                            let _ = departments::handle_dept(
                                cmd,
                                ctx.storage.clone(),
                                &departments::EngineRefs {
                                    code: None,
                                    content: None,
                                    harvest: None,
                                },
                            )
                            .await;
                        }
                    }
                    other => {
                        println!("Unknown command: {other}");
                        println!("Type 'help' for available commands.");
                    }
                }
            }
            Ok(Signal::CtrlC) => {
                println!("\nUse 'exit' to quit.");
            }
            Ok(Signal::CtrlD) => {
                println!("Goodbye.");
                break;
            }
            Err(e) => {
                eprintln!("Shell error: {e}");
                break;
            }
        }
    }

    Ok(())
}

fn dept_to_cmd(dept: &str, action: departments::DeptAction) -> Option<departments::DeptCmd> {
    Some(match dept {
        "finance" => departments::DeptCmd::Finance { action },
        "growth" => departments::DeptCmd::Growth { action },
        "distro" => departments::DeptCmd::Distro { action },
        "legal" => departments::DeptCmd::Legal { action },
        "support" => departments::DeptCmd::Support { action },
        "infra" => departments::DeptCmd::Infra { action },
        "product" => departments::DeptCmd::Product { action },
        "code" => departments::DeptCmd::Code { action },
        "harvest" => departments::DeptCmd::Harvest { action },
        "content" => departments::DeptCmd::Content { action },
        "gtm" => departments::DeptCmd::Gtm { action },
        _ => return None,
    })
}

fn print_help() {
    println!(
        r"
RUSVEL Interactive Shell
════════════════════════

Navigation:
  use <dept>          Switch into a department context
  back / ..           Leave department context
  exit / quit / q     Exit the shell

Session:
  session list         List all sessions
  session create NAME  Create a new session
  session switch ID    Switch active session

Departments:
  finance   growth   distro   legal    support
  infra     product  code     harvest  content   gtm

  <dept> status       Show department summary
  <dept> list [kind]  List items (optionally by kind)
  <dept> events       Show recent department events

Other:
  status              Show all departments summary
  dashboard           Info about TUI dashboard
  help / ?            Show this help

Tab for autocomplete. Ctrl+R to search history.
"
    );
}
