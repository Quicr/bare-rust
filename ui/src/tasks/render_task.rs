use super::{Task, TaskData};
use crate::metrics::Metrics;
use crate::msg::Msg;
use crate::tasks::TaskInfo;
use bsp::board;

use crate::font;

/// Structure representing the render task.
pub struct RenderTask {}

const FONT_WIDTH: usize = 11;
const FONT_HEIGHT: usize = 16;
const DISPLAY_WIDTH: usize = board::info::DISP_NUM_COLS;
const DISPLAY_HEIGHT: usize = board::info::DISP_NUM_ROWS;
const DISPLAY_BAND_HEIGHT: usize = 320 / 10;
const TEXT_ROWS: usize = DISPLAY_HEIGHT / FONT_HEIGHT;
const TEXT_COLS: usize = DISPLAY_WIDTH / FONT_WIDTH;

pub struct Data {
    text: [[u8; TEXT_COLS]; TEXT_ROWS],
    dirty: [bool; TEXT_ROWS],

    bitmap: [u16; DISPLAY_WIDTH * DISPLAY_BAND_HEIGHT],
    current_band: usize,
}

impl Data {
    /// Creates a new `Data` instance with an empty buffer.
    pub const fn new() -> Self {
        Data {
            text: [[0; TEXT_COLS]; TEXT_ROWS],
            dirty: [true; TEXT_ROWS],
            bitmap: [0; (DISPLAY_WIDTH * DISPLAY_BAND_HEIGHT)],
            current_band: 0,
        }
    }
}

/// Information about the render task.
const RENDER_TASK_INFO: TaskInfo = TaskInfo {
    name: b"Render__",
    run_every_us: 50_000,
    time_budget_us: 15_000,
    mem_budget_bytes: 500,
};

pub fn recv(
    msg: &Msg,
    _sender: &mut crate::mpsc::Sender<Msg>,
    _bsp: &mut bsp::BSP,
    task_data: &mut TaskData,
    _metrics: &mut Metrics,
) {
    let data = &mut task_data.render;

    match msg {
        Msg::PrintMsg { text } => {
            assert!(TEXT_ROWS > 3);
            // scroll the text up
            for r in 0..TEXT_ROWS - 3 {
                for c in 0..TEXT_COLS {
                    data.text[r][c] = data.text[r + 1][c];
                }
                data.dirty[r] = true;
            }

            // add new line
            let row = TEXT_ROWS - 2;
            let mut num_cols = text.len();
            if num_cols > TEXT_COLS {
                num_cols = TEXT_COLS;
            }
            for col in 0..num_cols {
                data.text[row][col] = text[col];
            }
            for col in num_cols..TEXT_COLS {
                data.text[row][col] = b' ';
            }
            data.dirty[row] = true;
        }
        Msg::PrintInputMsg { text } => {
            assert!(TEXT_ROWS > 1);
            let row = TEXT_ROWS - 1;
            let mut num_cols = text.len();
            if num_cols > TEXT_COLS {
                num_cols = TEXT_COLS;
            }
            for col in 0..num_cols {
                data.text[row][col] = text[col];
            }
            for col in num_cols..TEXT_COLS {
                data.text[row][col] = b' ';
            }
            data.dirty[row] = true;
        }
        Msg::PrintClearMsg {} => {
            for r in 0..TEXT_ROWS - 1 {
                for c in 0..TEXT_COLS {
                    data.text[r][c] = b' ';
                }
                data.dirty[r] = true;
            }
        }
        Msg::PrintClearInputMsg {} => {
            let r = TEXT_ROWS - 1;
            for c in 0..TEXT_COLS {
                data.text[r][c] = b' ';
            }
            data.dirty[r] = true;
        }
        _ => {
            // Handle other messages if necessary
        }
    }
}

fn render_glyph(c: u8, x: i32, y: i32, data: &mut Data) {
    let index = font::GLYPH_INDEX[c as usize] as usize;

    if index == 255 {
        return; // TOOO render black box.
    }

    let glyph = &font::GLYPH_METRICS[index];

    assert!(glyph.width >= font::METRICS.min_width);
    assert!(glyph.width <= font::METRICS.max_width);

    assert_eq!(c, glyph.c as u8);

    let mut rle_index = glyph.rle_start as usize;

    let mut row: i32 = y
        - (DISPLAY_HEIGHT as i32 - (data.current_band + 1) as i32 * DISPLAY_BAND_HEIGHT as i32)
        - (glyph.ymin as i32)
        - (glyph.height as i32)
        + 1;
    let mut col: i32 = x + glyph.xmin as i32;

    if row >= DISPLAY_BAND_HEIGHT as i32 {
        return;
    }
    if (row + glyph.height as i32) < 0 {
        return;
    }
    if col > DISPLAY_WIDTH as i32 {
        return;
    }
    if (col + glyph.width as i32) < 0 {
        return;
    }

    loop {
        assert!(rle_index < font::RLE_DATA.len());
        assert!(rle_index < font::METRICS.rle_bytes as usize);

        let (count, val) = font::RLE_DATA[rle_index];
        if count == 0 && val == 0 {
            break;
        }

        if count == -1 {
            row += 1;

            col = x + glyph.xmin as i32;
        }

        if count > 0 {
            let red: u16 = (val as u16) >> (8 - 5);
            let green: u16 = (val as u16) >> (8 - 6);
            let blue: u16 = (val as u16) >> (8 - 5);
            let color: u16 = (red << 11) | (green << 5) | blue;

            for _ in 0..count {
                if val < 255 {
                    if (row >= 0)
                        && (row < DISPLAY_BAND_HEIGHT as i32)
                        && (col >= 0)
                        && (col < DISPLAY_WIDTH as i32)
                    {
                        data.bitmap[(row * (DISPLAY_WIDTH as i32) + col) as usize] = color;
                    }
                }
                col += 1;
            }
        }

        rle_index += 1;
    }
}

impl Task for RenderTask {
    /// Method to execute the render task.
    /// Reads the state of the render and sends a message if the state has changed.
    fn run(
        &self,
        _sender: &mut crate::mpsc::Sender<Msg>,
        bsp: &mut bsp::BSP,
        task_data: &mut TaskData,
        _metrics: &mut Metrics,
    ) {
        let data = &mut task_data.render;

        if data.current_band == 0 {
            data.current_band = DISPLAY_HEIGHT / DISPLAY_BAND_HEIGHT - 1;
        } else {
            data.current_band -= 1;
        }

        for y in 0..DISPLAY_BAND_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                data.bitmap[y * DISPLAY_WIDTH + x] = 0xFFFF; // white
            }
        }

        if false {
            for r in 0..TEXT_ROWS {
                for c in 0..TEXT_COLS {
                    data.text[r][c] = 0;
                }
                data.dirty[r] = true;
            }
        }

        if false {
            for r in 0..5 {
                for c in 0..10 {
                    data.text[r][c] = '0' as u8 + (c % 10) as u8;
                }
                data.dirty[r] = true;
            }
        }

        let mut x: i32 = 0;
        let mut y: i32 = font::METRICS.ascent as i32;

        if true {
            for row in 0..TEXT_ROWS {
                for col in 0..TEXT_COLS {
                    let c = data.text[row][col];

                    let index = font::GLYPH_INDEX[c as usize] as usize;

                    if index == 255 {
                        continue;
                    }

                    let glyph = &font::GLYPH_METRICS[index];

                    render_glyph(c as u8, x, y, data);

                    x += glyph.width as i32 + 1;
                }
                y += font::METRICS.line_gap as i32 + font::METRICS.ascent as i32
                    - font::METRICS.descent as i32;
                x = 0;
            }
        }

        if false {
            let text = "Hello, AV!";
            for c in text.bytes() {
                render_glyph(c, x, y, data);
                x += 1 + font::GLYPH_METRICS[font::GLYPH_INDEX[c as usize] as usize].width as i32;
            }
        }

        bsp.display.draw_bitmap(
            &data.bitmap,
            0,                                       // x
            data.current_band * DISPLAY_BAND_HEIGHT, //y
            DISPLAY_WIDTH,
            DISPLAY_BAND_HEIGHT,
        );
    }

    /// Returns the information about the render task.
    fn info(&self) -> &'static TaskInfo {
        &RENDER_TASK_INFO
    }
}
