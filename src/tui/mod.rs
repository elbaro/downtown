pub mod code;
mod highlight;
pub mod raw_input;
pub mod scroll;
pub mod widgets;
pub type LineNum = usize;
pub type Terminal = ratatui::Terminal<CrosstermBackend<std::io::Stdout>>;

use crate::{
    cli::Config,
    profiler::{
        hist::{format_ns, Summary},
        ProfilerCommand,
    },
    tui::{code::PythonCode, scroll::Scroll},
};
use color_eyre::{
    eyre::{Context as ContextTrait, ContextCompat},
    Result,
};
use comfy_table::{presets::UTF8_FULL_CONDENSED, CellAlignment};
use crossterm::event::KeyCode;
use ratatui::{backend::CrosstermBackend, widgets::Clear};
use std::{collections::HashMap, path::Path};
use xtra::prelude::*;

#[derive(Debug)]
pub enum TuiMessage {
    KeyEvent(KeyCode),
    Render,
    Update,
    // Profile(HashMap<LineNum, Summary>),
}

#[derive(Actor)]
pub struct Tui {
    header_state: widgets::header::HeaderState,
    terminal: Terminal,
    code: PythonCode,
    highlighted: Vec<String>,
    _title: String,

    // state
    profiler_addr: Address<crate::profiler::Profiler>,
    scroll: Scroll,
    summary_map: HashMap<LineNum, Summary>,
}

fn setup_terminal() -> Result<Terminal> {
    crossterm::terminal::enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn cleanup_terminal(terminal: &mut Terminal) -> Result<()> {
    crossterm::terminal::disable_raw_mode().context("failed to disable raw mode")?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

impl Tui {
    pub fn new(
        config: &Config,
        code: PythonCode,
        profiler_addr: Address<crate::profiler::Profiler>,
    ) -> Result<Self> {
        let terminal = setup_terminal()?;

        let highlighted: Vec<String> = highlight::highlight(&code.source);
        let title = {
            let filename = Path::new(&config.python_code)
                .file_name()
                .context("no filename")?
                .to_str()
                .context("non-utf8 filename")?
                .to_string();
            let dir = Path::new(&config.python_code)
                .parent()
                .context("no parent dir")?
                .to_str()
                .context("non-utf8 parent dir")?
                .to_string();
            format!("{filename} ({dir}/)")
        };
        let num_lines = highlighted.len();

        let header_state = widgets::header::HeaderState {
            pid: if config.pid == -1 {
                "ALL".to_string()
            } else {
                config.pid.to_string()
            },
            python_bin: config.python_bin.clone(),
            code_path: config.python_code.clone(),
        };

        Ok(Self {
            profiler_addr,
            terminal,
            code,
            highlighted,
            _title: title,
            summary_map: Default::default(),
            scroll: Scroll::new(num_lines, 10),
            header_state,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            frame.render_widget(Clear, frame.size());

            // render parameters
            let mut area = frame.size();
            area.height = 10;
            let mut table = comfy_table::Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);

            table.add_row(["pid", &self.header_state.pid]);
            table.add_row(["python", &self.header_state.python_bin]);
            table.add_row(["code", &self.header_state.code_path]);
            frame.render_widget(
                widgets::table::Table {
                    table,
                    highlight_row: 999,
                },
                area,
            );
            // render main table
            let mut area = frame.size();
            area.y += 5;
            area.height -= 5;
            let mut table = comfy_table::Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_header(vec![
                "min", "p50", "p90", "p99", "max", "samples", "line", "code",
            ]);
            table
                .column_mut(6)
                .unwrap()
                .set_cell_alignment(CellAlignment::Right);

            self.scroll.resize_height(area.height as usize - 4);
            for i in self.scroll.range() {
                if let Some(summary) = self.summary_map.get(&i) {
                    table.add_row([
                        format_ns(summary.min),
                        format_ns(summary.p50),
                        format_ns(summary.p90),
                        format_ns(summary.p99),
                        format_ns(summary.max),
                        summary.samples.to_string(),
                        i.to_string(),
                        self.highlighted[i - 1].clone(),
                    ]);
                } else {
                    let ii = i.to_string();
                    table.add_row(["", "", "", "", "", "", &ii, &self.highlighted[i - 1]]);
                }
            }
            let table = widgets::table::Table {
                table,
                highlight_row: self.scroll.current_line - self.scroll.scroll_offset,
            };
            frame.render_widget(table, area);
        })?;
        Ok(())
    }

    async fn handle_impl(&mut self, msg: TuiMessage, ctx: &mut Context<Tui>) -> Result<()> {
        match msg {
            TuiMessage::KeyEvent(ev) => match ev {
                KeyCode::Char('q') => {
                    ctx.stop_self();
                }
                KeyCode::Up => {
                    self.scroll.up(1);
                    self.render()?;
                }
                KeyCode::Down => {
                    self.scroll.down(1);
                    self.render()?;
                }
                KeyCode::PageUp => {
                    if let Some(line) = self.code.jump_to_prev_fn(self.scroll.current_line) {
                        self.scroll.up(self.scroll.current_line - line);
                        self.render()?;
                    }
                }
                KeyCode::PageDown => {
                    if let Some(line) = self.code.jump_to_next_fn(self.scroll.current_line) {
                        self.scroll.down(line - self.scroll.current_line);
                        self.render()?;
                    }
                }
                KeyCode::Enter => {
                    self.profiler_addr
                        .send(ProfilerCommand::ToggleFilter(
                            crate::profiler::types::Filter {
                                lineno: self.scroll.current_line, // convert to 1-based
                            },
                        ))
                        .await
                        .context("failed to send ProfilerMessage::ToggleFilter")??;
                }
                _ => unreachable!(),
            },
            TuiMessage::Render => self.render()?,
            TuiMessage::Update => {
                let summary_map = self
                    .profiler_addr
                    .send(crate::profiler::request::Observe)
                    .await??;
                self.summary_map = summary_map;
                self.render()?;
            }
        }
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        if let Err(e) = cleanup_terminal(&mut self.terminal) {
            log::error!("Failed to clean up raw terminal: {}", e);
        }
    }
}

#[async_trait::async_trait]
impl Handler<TuiMessage> for Tui {
    type Return = ();
    async fn handle(&mut self, msg: TuiMessage, ctx: &mut Context<Self>) {
        if let Err(e) = self.handle_impl(msg, ctx).await {
            log::error!("{}", e);
        }
    }
}
