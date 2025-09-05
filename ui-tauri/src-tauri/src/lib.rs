// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::ops::DerefMut;
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tauri::Emitter;
use ui_app::{App, Event, Led, Outputs};

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
}

const UI_LED_NAME: &str = "led-ui";

#[derive(Debug)]
struct Board {
    app: tauri::AppHandle,
}

impl Board {
    fn new(app: tauri::AppHandle) -> Self {
        Self { app }
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

impl Outputs for Board {
    fn status_led(&mut self) -> &mut impl Led {
        self
    }
}

#[tauri::command]
fn start() {
    println!("Start");
    let mut board = BOARD.get().unwrap().lock().unwrap();

    let ui_app = App::start(board.deref_mut());
    UI_APP.set(Mutex::new(ui_app)).unwrap();
}

#[tauri::command]
fn ptt_button_press(name: &str) {
    println!("PTT button event: {}", name);
    let mut board = BOARD.get().unwrap().lock().unwrap();
    let mut ui_app = UI_APP.get().unwrap().lock().unwrap();
    match name {
        "mousedown" => ui_app.handle(Event::PttDown, board.deref_mut()),
        "mouseup" => ui_app.handle(Event::PttUp, board.deref_mut()),
        _ => {}
    }
}

#[tauri::command]
fn ai_button_press(name: &str) {
    println!("AI button press");
    let mut board = BOARD.get().unwrap().lock().unwrap();
    let mut ui_app = UI_APP.get().unwrap().lock().unwrap();
    match name {
        "mousedown" => ui_app.handle(Event::AiDown, board.deref_mut()),
        "mouseup" => ui_app.handle(Event::AiUp, board.deref_mut()),
        _ => {}
    }
}

#[tauri::command]
fn keydown(code: &str) {
    // TODO actually handle
    println!("keydown: {}", code);
}

#[tauri::command]
fn keyup(code: &str) {
    // TODO actually handle
    println!("keyup: {}", code);
}

static BOARD: OnceCell<Mutex<Board>> = OnceCell::new();
static UI_APP: OnceCell<Mutex<App>> = OnceCell::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let board = Board::new(app.handle().clone());
            BOARD.set(Mutex::new(board)).unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start,
            ai_button_press,
            ptt_button_press,
            keydown,
            keyup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
