use ui_app::*;

#[derive(Default)]
struct MockLed;

impl Led for MockLed {
    fn set(&mut self, r: bool, g: bool, b: bool) {
        let r = if r { 'R' } else { '_' };
        let g = if g { 'G' } else { '_' };
        let b = if b { 'B' } else { '_' };
        println!("{}{}{}", r, g, b);
    }
}

#[derive(Default)]
struct MockTx;

impl ui_app::Write for MockTx {
    fn write(&mut self, _buf: &[u8]) -> usize {
        0
    }
}

#[derive(Default)]
struct MockOutputs {
    status_led: MockLed,
    mgmt_tx: MockTx,
}

impl Outputs for MockOutputs {
    fn status_led(&mut self) -> &mut impl Led {
        &mut self.status_led
    }

    fn mgmt_tx(&mut self) -> &mut impl ui_app::Write {
        &mut self.mgmt_tx
    }
}

use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    let mut board = MockOutputs::default();
    let mut app = App::start(&mut board);

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(
        stdout,
        "{}{}Ctrl-P for PTT.  Ctrl-A for AI.  Q to quit.{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();
    stdout.flush().unwrap();

    let mut ptt_down = false;
    let mut ai_down = false;

    for k in stdin.keys() {
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 2),
            termion::clear::CurrentLine
        )
        .unwrap();

        match k.as_ref().unwrap() {
            Key::Char('q') => break,
            Key::Ctrl('p') => {
                ptt_down = !ptt_down;
                if ptt_down {
                    app.handle(Event::PttDown, &mut board);
                } else {
                    app.handle(Event::PttUp, &mut board);
                }
            }
            Key::Ctrl('a') => {
                ai_down = !ai_down;
                if ai_down {
                    app.handle(Event::AiDown, &mut board);
                } else {
                    app.handle(Event::AiUp, &mut board);
                }
            }
            _ => {}
        }
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
