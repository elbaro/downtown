use crossterm::event::{Event, KeyCode};
use futures::{Stream, StreamExt};

use crate::tui::TuiMessage;

pub fn term_input_stream() -> impl Stream<Item = Result<TuiMessage, xtra::Error>> {
    let stream = crossterm::event::EventStream::new();
    stream.filter_map(|ev| async move {
        match ev {
            Ok(ev) => match ev {
                Event::Key(ev) => match ev.code {
                    KeyCode::Char('q')
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::Enter => Some(Ok(TuiMessage::KeyEvent(ev.code))),
                    _ => None,
                },
                _ => None,
            },
            Err(err) => {
                log::error!("{}", err);
                None
            }
        }
    })
}
