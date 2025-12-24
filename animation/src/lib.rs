use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::Hide,
    cursor::Show,
    style::{Color, Print, ResetColor, SetForegroundColor},
    event::{self, Event, KeyCode, KeyModifiers},
};
use rand::Rng;
use std::io::stdout;
use std::time::Duration as StdDuration;

// Animation configuration constants
const BASE_HEAT: i32 = 40;
const HEAT_SCALING_FACTOR: i32 = 3;
const MAX_HEAT_SCALING: i32 = 70;
const BASE_INJECTIONS_DIV: usize = 4;
const EVENT_INJECTIONS_DIV: usize = 8;
const COOLING_EVENT_FACTOR: i32 = 30;

pub fn run_animation(contribs: bool, msg_text: String, meta_text: String, have_ticker: bool, speed: u8, num_events: usize, smoke_factor: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    enable_raw_mode()?;

    let (width, height) = crossterm::terminal::size()?;
    let size = (width as usize) * (height as usize);
    let mut buffer = vec![0i32; size + width as usize + 1];

    let chars: Vec<char>;
    let colors: Vec<Color>;

    if contribs {
        // GitHub contribution graph-style
        chars = vec![' ', ' ', '⬝', '⯀', '⯀', '◼', '◼', '■', '■', '■'];
        colors = vec![
            Color::Black,
            Color::Rgb { r: 155, g: 233, b: 168 }, // #9be9a8
            Color::Rgb { r: 64, g: 196, b: 99 },   // #40c463
            Color::Rgb { r: 48, g: 161, b: 78 },   // #30a14e
            Color::Rgb { r: 33, g: 110, b: 57 },   // #216e39
        ];
    } else {
        // Fire style with 5 heat levels
        chars = vec![' ', ' ', ' ', ':', '^', '*', 'x', 's', 'S', '#', '$'];
        colors = vec![
            Color::Black,                           // No heat
            Color::Rgb { r: 135, g: 206, b: 235 }, // Sky blue (low heat)
            Color::Blue,                            // Blue
            Color::Yellow,                          // Yellow
            Color::Rgb { r: 255, g: 165, b: 0 },   // Orange
            Color::Red,                             // Red (hottest)
        ];
    }

    let msg_row = height as usize - 2;
    let meta_row = height as usize - 1;
    let mut ticker_offset = 0;
    let mut frame = 0;

    loop {
        // Check for input
        if event::poll(StdDuration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc ||
                   (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)) {
                    break;
                }
            }
        }

        // Inject heat
        let mut rng = rand::thread_rng();
        let heat_scaling = (num_events as i32 * HEAT_SCALING_FACTOR).min(MAX_HEAT_SCALING);
        let heat_base = BASE_HEAT + heat_scaling;
        let base_injections = (width as usize) / BASE_INJECTIONS_DIV;
        let event_injections = num_events / EVENT_INJECTIONS_DIV;
        let num_injections = base_injections + event_injections;
        for _ in 0..num_injections {
            let idx = rng.gen_range(0..width) + width * (height - 1);
            if (idx as usize) < buffer.len() {
                buffer[idx as usize] = heat_base;
            }
        }

        // Propagate and cool
        let mut new_buffer = vec![0i32; buffer.len()];
        let smoke_level = smoke_factor as i32;
        for y in (0..height as usize).rev() {
            for x in 0..width as usize {
                let idx = x + y * width as usize;
                let below = if y < height as usize - 1 { buffer[x + (y + 1) * width as usize] } else { 0 };
                let below_right = if y < height as usize - 1 && x < width as usize - 1 { buffer[x + 1 + (y + 1) * width as usize] } else { 0 };
                let left = if x > 0 { buffer[x - 1 + y * width as usize] } else { 0 };
                let right = if x < width as usize - 1 { buffer[x + 1 + y * width as usize] } else { 0 };
                let avg = (left + right + below + below_right) / 4;
                let cooling = (speed as i32 - (num_events as i32 / COOLING_EVENT_FACTOR)).max(1);
                new_buffer[idx] = (avg - cooling).max(smoke_level);
            }
        }
        buffer = new_buffer;

        // Draw
        for i in 0..size {
            let v = buffer[i];
            let row = i / width as usize;
            let col = i % width as usize;

            if row >= height as usize || col >= width as usize {
                continue;
            }

            // Reserve bottom lines for ticker
            if have_ticker && row >= height as usize - 2 {
                continue;
            }

            // Clear top rows for fire animation to show empty space above smoke
            if !contribs && row < 5 {
                execute!(
                    stdout,
                    crossterm::cursor::MoveTo(col as u16, row as u16),
                    Print(' ')
                )?;
                continue;
            }

            let color_idx = match v {
                v if v > 20 => 5, // Red (hottest)
                v if v > 15 => 4, // Orange
                v if v > 10 => 3, // Yellow
                v if v > 5 => 2,  // Blue
                v if v > 0 => 1,  // Sky blue
                _ => 0,            // Black (no heat)
            };

            let ch_idx = ((v as usize * chars.len()) / 25).min(chars.len() - 1);

            execute!(
                stdout,
                crossterm::cursor::MoveTo(col as u16, row as u16),
                SetForegroundColor(colors[color_idx]),
                Print(chars[ch_idx])
            )?;
        }

        // Draw ticker
        if have_ticker && height >= 2 && !msg_text.is_empty() {
            let msg_chars: Vec<char> = msg_text.chars().collect();
            let meta_chars: Vec<char> = meta_text.chars().collect();
            let msg_len = msg_chars.len();
            let meta_len = meta_chars.len();

            if msg_len > 0 && meta_len > 0 {
                for x in 0..width as usize {
                    let mi = (ticker_offset + x) % msg_len;
                    let mj = (ticker_offset + x) % meta_len;
                    let mr = msg_chars[mi];
                    let me = meta_chars[mj];

                    execute!(
                        stdout,
                        crossterm::cursor::MoveTo(x as u16, msg_row as u16),
                        SetForegroundColor(Color::White),
                        Print(mr)
                    )?;

                    execute!(
                        stdout,
                        crossterm::cursor::MoveTo(x as u16, meta_row as u16),
                        SetForegroundColor(Color::White),
                        Print(me)
                    )?;
                }

                if frame % (11 - speed as usize) == 0 {
                    ticker_offset = (ticker_offset + speed as usize) % msg_len;
                }
            }
        }

        execute!(stdout, ResetColor)?;
        std::thread::sleep(StdDuration::from_millis(30));
        frame += 1;
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;
    Ok(())
}