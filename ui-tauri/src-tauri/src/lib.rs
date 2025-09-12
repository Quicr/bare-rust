// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::ops::DerefMut;
use once_cell::sync::OnceCell;
use std::sync::{mpsc, Mutex};
use tauri::Emitter;
use ui_app::{App, Button, Event, FromNet, Key, KeyValue, Led, NetTx, Outputs, Screen, ToNet};

mod hactar_vaporwave;

mod cmd {
    use serde_derive::Serialize;

    #[derive(Clone, Serialize)]
    pub struct Led {
        pub name: &'static str,
        pub r: usize,
        pub g: usize,
        pub b: usize,
    }

    #[derive(Clone, Serialize)]
    pub struct Draw<'a> {
        pub left: usize,
        pub right: usize,
        pub top: usize,
        pub bottom: usize,

        /// Data in <canvas> RGBA format
        pub data: &'a [u8],
    }
}

const UI_LED_NAME: &str = "led-ui";

#[derive(Debug)]
struct Board {
    app: tauri::AppHandle,
    to_net: mpsc::Sender<ToNet>,
}

impl Board {
    fn new(app: tauri::AppHandle, to_net: mpsc::Sender<ToNet>) -> Self {
        Self { app, to_net }
    }
}

impl Led for Board {
    fn set(&mut self, r: bool, g: bool, b: bool) {
        let command = cmd::Led {
            name: UI_LED_NAME,
            r: if r { 0xff } else { 0x00 },
            g: if g { 0xff } else { 0x00 },
            b: if b { 0xff } else { 0x00 },
        };

        self.app.emit("LED", command).unwrap();
    }
}

impl Screen for Board {
    fn width(&self) -> usize {
        320
    }

    fn height(&self) -> usize {
        240
    }

    fn fill(&mut self, color: u16) {
        let data = [color; 320 * 240];
        self.draw(0, self.width(), 0, self.height(), &data);
    }

    fn draw(&mut self, left: usize, right: usize, top: usize, bottom: usize, data: &[u16]) {
        println!(
            "draw {} {} {} {} ({} x {} == {}? {})",
            left,
            right,
            top,
            bottom,
            (right - left),
            (bottom - top),
            data.len(),
            data.len() == (right - left) * (bottom - top)
        );

        // Unpack the rgb565 values to RGBA tuples
        let data: Vec<u8> = data
            .iter()
            .map(|rgb565| {
                [
                    (((rgb565 & 0b11111_000000_00000) >> 11) << 3) as u8, // R
                    (((rgb565 & 0b00000_111111_00000) >> 5) << 2) as u8,  // G
                    (((rgb565 & 0b00000_000000_11111) >> 0) << 3) as u8,  // B
                    (0xff as u8),                                         // A
                ]
            })
            .flatten()
            .collect();

        // Send the draw command to the UI
        let cmd = cmd::Draw {
            left,
            right,
            top,
            bottom,
            data: data.as_slice(),
        };

        self.app.emit("Screen", cmd).unwrap();
    }
}

impl NetTx for Board {
    fn write(&mut self, to_net: &ToNet) {
        self.to_net.send(*to_net).unwrap();
    }
}

impl Outputs for Board {
    fn status_led(&mut self) -> &mut impl Led {
        self
    }

    fn screen(&mut self) -> &mut impl Screen {
        self
    }

    fn net_tx(&mut self) -> &mut impl NetTx {
        self
    }

    fn log(&mut self, message: &str) {
        println!("LOG: {}", message);
    }
}

#[tauri::command]
fn start() {
    println!("Start");
    let mut board = BOARD.get().unwrap().lock().unwrap();
    let mut ui_app = UI_APP.get().unwrap().lock().unwrap();
    ui_app.start(board.deref_mut());
}

#[tauri::command]
fn button_a_press(name: &str) {
    const DOWN: Event = Event::ButtonDown(Button::A);
    const UP: Event = Event::ButtonUp(Button::A);

    println!("Button A event: {}", name);
    let mut board = BOARD.get().unwrap().lock().unwrap();
    let mut ui_app = UI_APP.get().unwrap().lock().unwrap();
    match name {
        "mousedown" => ui_app.handle(DOWN, board.deref_mut()),
        "mouseup" => ui_app.handle(UP, board.deref_mut()),
        _ => {}
    }
}

#[tauri::command]
fn button_b_press(name: &str) {
    const DOWN: Event = Event::ButtonDown(Button::B);
    const UP: Event = Event::ButtonUp(Button::B);

    println!("Button B event: {}", name);
    let mut board = BOARD.get().unwrap().lock().unwrap();
    let mut ui_app = UI_APP.get().unwrap().lock().unwrap();
    match name {
        "mousedown" => ui_app.handle(DOWN, board.deref_mut()),
        "mouseup" => ui_app.handle(UP, board.deref_mut()),
        _ => {}
    }
}

fn key_from_code(code: usize) -> Key {
    match code {
        65 => Key::A,
        66 => Key::B,
        67 => Key::C,
        68 => Key::D,
        69 => Key::E,
        70 => Key::F,
        71 => Key::G,
        72 => Key::H,
        73 => Key::I,
        74 => Key::J,
        75 => Key::K,
        76 => Key::L,
        77 => Key::M,
        78 => Key::N,
        79 => Key::O,
        80 => Key::P,
        81 => Key::Q,
        82 => Key::R,
        83 => Key::S,
        84 => Key::T,
        85 => Key::U,
        86 => Key::V,
        87 => Key::W,
        88 => Key::X,
        89 => Key::Y,
        90 => Key::Z,
        8 => Key::Backspace,
        18 => Key::Alt,
        52 => Key::Dollar,
        13 => Key::Enter,
        16 => Key::LeftShift,
        91 => Key::Mic,
        32 => Key::Space,
        19 => Key::Sym,
        17 => Key::RightShift,
        _ => unreachable!(),
    }
}

fn key_value_from_string(value: &str) -> KeyValue {
    if value.len() == 1 {
        return KeyValue::Char(value.chars().next().unwrap());
    }

    match value {
        "Backspace" => KeyValue::Backspace,
        "AltLeft" => KeyValue::Alt,
        "🔊" => KeyValue::Speaker,
        "Enter" => KeyValue::Enter,
        "ShiftLeft" => KeyValue::LeftShift,
        "🎤︎" => KeyValue::Mic,
        "Space" => KeyValue::Space,
        "AltRight" => KeyValue::Sym,
        "ShiftRight" => KeyValue::RightShift,
        _ => unreachable!(),
    }
}

#[tauri::command]
fn keydown(code: usize, value: &str) {
    let mut board = BOARD.get().unwrap().lock().unwrap();
    let mut ui_app = UI_APP.get().unwrap().lock().unwrap();

    let key = key_from_code(code);
    let value = key_value_from_string(value);

    println!("keydown: {:?} {:?} {:?}", code, key, value);
    ui_app.handle(Event::KeyDown(key, value), board.deref_mut());
}

#[tauri::command]
fn keyup(code: usize) {
    let mut board = BOARD.get().unwrap().lock().unwrap();
    let mut ui_app = UI_APP.get().unwrap().lock().unwrap();

    let key = key_from_code(code);

    println!("keyup: {:?}", key);
    ui_app.handle(Event::KeyUp(key), board.deref_mut());
}

static BOARD: OnceCell<Mutex<Board>> = OnceCell::new();
static UI_APP: OnceCell<Mutex<App>> = OnceCell::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (to_net, from_ui) = mpsc::channel::<ToNet>();
    let (to_ui, from_net) = mpsc::channel::<FromNet>();

    // Consume and process NET messages
    std::thread::spawn(move || {
        while let Ok(to_net) = from_ui.recv() {
            match to_net {
                ToNet::Ping => {
                    to_ui.send(FromNet::Pong).unwrap();
                }
            }
        }
    });

    // Forward messages to NET to the app
    std::thread::spawn(move || {
        while let Ok(from_net) = from_net.recv() {
            let mut board = BOARD.get().unwrap().lock().unwrap();
            let mut ui_app = UI_APP.get().unwrap().lock().unwrap();
            ui_app.handle(Event::FromNet(from_net), board.deref_mut());
        }
    });

    tauri::Builder::default()
        .setup(|app| {
            let board = Board::new(app.handle().clone(), to_net);
            BOARD.set(Mutex::new(board)).unwrap();

            let ui_app = App::new();
            UI_APP.set(Mutex::new(ui_app)).unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start,
            button_a_press,
            button_b_press,
            keydown,
            keyup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
