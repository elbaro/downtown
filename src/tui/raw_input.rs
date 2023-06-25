use color_eyre::Result;
use crossterm::event::{Event, KeyCode};
use futures::StreamExt;
use tonari_actor::Addr;

use crate::{
    error::Error,
    tui::{Tui, TuiMessage},
};

pub async fn drain_term_input(tui_addr: Addr<Tui>) -> Result<()> {
    let mut stream = crossterm::event::EventStream::new();
    loop {
        let ev: Option<Result<_, _>> = stream.next().await;
        let ev = ev.ok_or(Error::InputStreamClosed)?;
        let ev = ev.map_err(|source| Error::InputStreamError { source })?;

        match ev {
            Event::Key(ev) => match ev.code {
                KeyCode::Char('q') => {
                    break;
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
    log::debug!("drain_term_input() finished");
    Ok(())
}
