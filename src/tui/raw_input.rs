use color_eyre::Result;
use crossterm::event::{Event, KeyCode};
use tonari_actor::{Addr, SystemHandle};

use crate::tui::{Tui, TuiMessage};

pub fn drain(system: SystemHandle, tui_addr: Addr<Tui>) -> Result<()> {
    loop {
        let ev = crossterm::event::read()?;
        match ev {
            Event::Key(ev) => match ev.code {
                KeyCode::Char('q') => {
                    system.shutdown()?;
                }
                KeyCode::Up => tui_addr.send(TuiMessage::ScrollUp)?,
                KeyCode::Down => tui_addr.send(TuiMessage::ScrollDown)?,
                KeyCode::PageUp => tui_addr.send(TuiMessage::PageUp)?,
                KeyCode::PageDown => tui_addr.send(TuiMessage::PageDown)?,
                KeyCode::Enter => tui_addr.send(TuiMessage::Enter)?,
                _ => {}
            },
            Event::Mouse(_ev) => {}
            Event::Resize(_w, _h) => {}
            _ => {}
        }
    }
    // Ok(())
}
