use anyhow::Context;
use clap::{Arg, Command};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    QueueableCommand,
};
use rand::Rng;
use std::io::{self, Stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};

static TERM_CLEANED: AtomicBool = AtomicBool::new(false);

struct Config {
    speed_scale: f64,
    density: f64,
    wind: f64,
    color_mode: ColorMode,
}

enum ColorMode {
    Gradient,
    Matrix,
    Nord,
    Gruvbox,
    Dracula,
    Catppuccin,
    Monokai,
    Solarized,
}

fn gradient_color(speed: u16) -> u8 {
    let x = speed as f64;
    let index = (0.0416 * (x - 4.0) * (x - 3.0) * (x - 2.0) - 4.0) * (x - 1.0) + 255.0;
    index.clamp(0.0, 255.0) as u8
}

fn nord_color(speed: u16) -> (u8, u8, u8) {
    match speed {
        1 | 2 => (0x5E, 0x81, 0xAC),
        3 => (0x81, 0xA1, 0xC1),
        4 | 5 => (0x88, 0xC0, 0xD0),
        6 => (0xD8, 0xDE, 0xE9),
        _ => (0x5E, 0x81, 0xAC),
    }
}

fn gruvbox_color(speed: u16) -> (u8, u8, u8) {
    match speed {
        1 | 2 => (0x8E, 0xC0, 0x7C),
        3 => (0xB8, 0xBB, 0x26),
        4 => (0xFA, 0xBD, 0x2F),
        5 | 6 => (0xFB, 0x49, 0x34),
        _ => (0x8E, 0xC0, 0x7C),
    }
}

fn dracula_color(speed: u16) -> (u8, u8, u8) {
    match speed {
        1 => (0x62, 0x72, 0xA4),
        2 => (0x8B, 0xE9, 0xFD),
        3 => (0x50, 0xFA, 0x7B),
        4 => (0xF1, 0xFA, 0x8C),
        5 => (0xFF, 0xB8, 0x6C),
        6 => (0xFF, 0x79, 0xC6),
        _ => (0x62, 0x72, 0xA4),
    }
}

fn catppuccin_color(speed: u16) -> (u8, u8, u8) {
    match speed {
        1 => (0x89, 0xB4, 0xFA),
        2 => (0x74, 0xC7, 0xEC),
        3 => (0x94, 0xE2, 0xD5),
        4 => (0xA6, 0xE3, 0xA1),
        5 => (0xF9, 0xE2, 0xAF),
        6 => (0xFA, 0xB3, 0x87),
        _ => (0x89, 0xB4, 0xFA),
    }
}

fn monokai_color(speed: u16) -> (u8, u8, u8) {
    match speed {
        1 => (0x66, 0xD9, 0xEF),
        2 => (0xA6, 0xE2, 0x2E),
        3 => (0xE6, 0xDB, 0x74),
        4 => (0xFD, 0x97, 0x1F),
        5 => (0xF9, 0x26, 0x72),
        6 => (0xAE, 0x81, 0xFF),
        _ => (0x66, 0xD9, 0xEF),
    }
}

fn solarized_color(speed: u16) -> (u8, u8, u8) {
    match speed {
        1 => (0x26, 0x8B, 0xD2),
        2 => (0x2A, 0xA1, 0x98),
        3 => (0x85, 0x99, 0x00),
        4 => (0xB5, 0x89, 0x00),
        5 => (0xD3, 0x36, 0x82),
        6 => (0x6C, 0x71, 0xC4),
        _ => (0x26, 0x8B, 0xD2),
    }
}

fn color_for_speed(speed: u16, mode: &ColorMode) -> Color {
    match mode {
        ColorMode::Gradient => Color::AnsiValue(gradient_color(speed)),
        ColorMode::Matrix => Color::Green,
        ColorMode::Nord => {
            let (r, g, b) = nord_color(speed);
            Color::Rgb { r, g, b }
        }
        ColorMode::Gruvbox => {
            let (r, g, b) = gruvbox_color(speed);
            Color::Rgb { r, g, b }
        }
        ColorMode::Dracula => {
            let (r, g, b) = dracula_color(speed);
            Color::Rgb { r, g, b }
        }
        ColorMode::Catppuccin => {
            let (r, g, b) = catppuccin_color(speed);
            Color::Rgb { r, g, b }
        }
        ColorMode::Monokai => {
            let (r, g, b) = monokai_color(speed);
            Color::Rgb { r, g, b }
        }
        ColorMode::Solarized => {
            let (r, g, b) = solarized_color(speed);
            Color::Rgb { r, g, b }
        }
    }
}

fn term_params(cols: u16, lines: u16, density: f64) -> (bool, u16) {
    let small = (lines < 20 && cols > 100) || (cols < 100 && lines < 40);
    let base = if small {
        cols as f64 * 0.75
    } else {
        cols as f64 * 1.5
    };
    let n = (base * density) as u16;
    (small, n)
}

struct Drop {
    w: u16,
    h: u16,
    speed: u16,
    color: Color,
    shape: char,
}

impl Drop {
    fn new(
        cols: u16,
        lines: u16,
        speed_scale: f64,
        color_mode: &ColorMode,
        small_term: bool,
    ) -> Self {
        let mut rng = rand::rng();
        let w = rng.random_range(0..cols);
        let h = rng.random_range(0..lines);
        let speed_raw = if small_term {
            rng.random_range(1..3u16)
        } else {
            rng.random_range(1..6u16)
        };
        let speed = ((speed_raw as f64 * speed_scale).round() as u16).max(1);
        let shape = if small_term {
            if speed_raw >= 2 {
                ':'
            } else {
                '|'
            }
        } else {
            if speed_raw >= 3 {
                ':'
            } else {
                '|'
            }
        };
        let color = color_for_speed(speed_raw, color_mode);
        Self {
            w,
            h,
            speed,
            color,
            shape,
        }
    }

    fn fall(&mut self, lines: u16, wind: i16, cols: u16) {
        self.h += self.speed;
        if wind != 0 && cols > 0 {
            self.w = (self.w as i16 + wind).rem_euclid(cols as i16) as u16;
        }
        if self.h >= lines.saturating_sub(1) {
            self.h = rand::rng().random_range(0..10);
            self.w = rand::rng().random_range(0..cols);
        }
    }

    fn render(&self, stdout: &mut Stdout) -> io::Result<()> {
        stdout
            .queue(MoveTo(self.w, self.h))?
            .queue(SetForegroundColor(self.color))?
            .queue(Print(self.shape))?;
        Ok(())
    }
}

fn valid_speed(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse().map_err(|_| "speed must be a number")?;
    if (0.5..=3.0).contains(&v) {
        Ok(v)
    } else {
        Err("speed must be between 0.5 and 3.0".into())
    }
}

fn valid_density(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse().map_err(|_| "density must be a number")?;
    if (0.1..=5.0).contains(&v) {
        Ok(v)
    } else {
        Err("density must be between 0.1 and 5.0".into())
    }
}

fn valid_wind(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse().map_err(|_| "wind must be a number")?;
    if (-5.0..=5.0).contains(&v) {
        Ok(v)
    } else {
        Err("wind must be between -5.0 and 5.0".into())
    }
}

fn init_terminal() -> anyhow::Result<Stdout> {
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide).context("failed to init terminal")?;
    Ok(stdout)
}

fn cleanup_terminal() {
    if TERM_CLEANED.swap(true, Ordering::Relaxed) {
        return;
    }
    let mut stdout = io::stdout();
    execute!(stdout, Show, LeaveAlternateScreen).ok();
    disable_raw_mode().ok();
}

struct TerminalGuard;

impl std::ops::Drop for TerminalGuard {
    fn drop(&mut self) {
        cleanup_terminal();
    }
}

struct App {
    drops: Vec<Drop>,
    config: Config,
    cols: u16,
    lines: u16,
}

impl App {
    fn new(config: Config, cols: u16, lines: u16) -> Self {
        let n = if cols == 0 || lines == 0 {
            0
        } else {
            term_params(cols, lines, config.density).1 as usize
        };
        let small = term_params(cols, lines, config.density).0;
        let drops = (0..n)
            .map(|_| {
                Drop::new(
                    cols.max(1),
                    lines.max(1),
                    config.speed_scale,
                    &config.color_mode,
                    small,
                )
            })
            .collect();
        Self {
            drops,
            config,
            cols,
            lines,
        }
    }

    fn resize(&mut self, cols: u16, lines: u16) {
        self.cols = cols;
        self.lines = lines;
        let n = if cols == 0 || lines == 0 {
            0
        } else {
            term_params(cols, lines, self.config.density).1 as usize
        };
        let small = term_params(cols, lines, self.config.density).0;
        self.drops.resize_with(n, || {
            Drop::new(
                cols.max(1),
                lines.max(1),
                self.config.speed_scale,
                &self.config.color_mode,
                small,
            )
        });
        self.drops.truncate(n);
    }

    fn run(&mut self, stdout: &mut Stdout) -> anyhow::Result<()> {
        use std::time::Duration;

        let wind = self.config.wind.round() as i16;

        loop {
            let mut resized = false;
            while event::poll(Duration::ZERO).context("event poll failed")? {
                match event::read().context("event read failed")? {
                    Event::Resize(w, h) if !resized => {
                        self.resize(w, h);
                        std::thread::sleep(Duration::from_millis(90));
                        resized = true;
                    }
                    Event::Resize(_, _) => {}
                    Event::Key(key)
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') =>
                    {
                        return Ok(());
                    }
                    _ => {}
                }
            }

            stdout.queue(Clear(ClearType::All))?;

            for drop in &mut self.drops {
                drop.fall(self.lines, wind, self.cols);
                drop.render(stdout)?;
            }
            stdout.flush()?;

            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

impl std::ops::Drop for App {
    fn drop(&mut self) {
        cleanup_terminal();
    }
}

fn main() -> anyhow::Result<()> {
    let matches = Command::new("rain-rs")
        .about("Terminal rain animation")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("speed")
                .short('s')
                .long("speed")
                .default_value("1.0")
                .value_parser(valid_speed),
        )
        .arg(
            Arg::new("density")
                .short('d')
                .long("density")
                .default_value("1.0")
                .value_parser(valid_density),
        )
        .arg(
            Arg::new("wind")
                .short('w')
                .long("wind")
                .default_value("0.0")
                .value_parser(valid_wind),
        )
        .arg(
            Arg::new("color-mode")
                .short('c')
                .long("color-mode")
                .default_value("gradient")
                .value_parser([
                    "gradient",
                    "matrix",
                    "nord",
                    "gruvbox",
                    "dracula",
                    "catppuccin",
                    "monokai",
                    "solarized",
                ]),
        )
        .get_matches();

    let config = Config {
        speed_scale: *matches.get_one::<f64>("speed").unwrap(),
        density: *matches.get_one::<f64>("density").unwrap(),
        wind: *matches.get_one::<f64>("wind").unwrap(),
        color_mode: match matches.get_one::<String>("color-mode").unwrap().as_str() {
            "gradient" => ColorMode::Gradient,
            "matrix" => ColorMode::Matrix,
            "nord" => ColorMode::Nord,
            "gruvbox" => ColorMode::Gruvbox,
            "dracula" => ColorMode::Dracula,
            "catppuccin" => ColorMode::Catppuccin,
            "monokai" => ColorMode::Monokai,
            "solarized" => ColorMode::Solarized,
            _ => unreachable!(),
        },
    };

    let mut stdout = init_terminal()?;
    let _guard = TerminalGuard;

    let (cols, lines) = crossterm::terminal::size().context("failed to get terminal size")?;

    let mut app = App::new(config, cols, lines);
    app.run(&mut stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_color_speed_1() {
        assert_eq!(gradient_color(1), 255);
    }

    #[test]
    fn test_gradient_color_speed_2() {
        assert_eq!(gradient_color(2), 251);
    }

    #[test]
    fn test_gradient_color_speed_3() {
        assert_eq!(gradient_color(3), 247);
    }

    #[test]
    fn test_gradient_color_speed_4() {
        assert_eq!(gradient_color(4), 243);
    }

    #[test]
    fn test_gradient_color_speed_5() {
        assert_eq!(gradient_color(5), 239);
    }

    #[test]
    fn test_gradient_color_speed_6() {
        assert_eq!(gradient_color(6), 239);
    }

    #[test]
    fn test_nord_color_speed_1() {
        let color = nord_color(1);
        assert_eq!(color, (0x5E, 0x81, 0xAC));
    }

    #[test]
    fn test_nord_color_speed_6() {
        let color = nord_color(6);
        assert_eq!(color, (0xD8, 0xDE, 0xE9));
    }

    #[test]
    fn test_gruvbox_color_speed_1() {
        let color = gruvbox_color(1);
        assert_eq!(color, (0x8E, 0xC0, 0x7C));
    }

    #[test]
    fn test_gruvbox_color_speed_6() {
        let color = gruvbox_color(6);
        assert_eq!(color, (0xFB, 0x49, 0x34));
    }

    #[test]
    fn test_dracula_color_speed_1() {
        assert_eq!(dracula_color(1), (0x62, 0x72, 0xA4));
    }

    #[test]
    fn test_dracula_color_speed_6() {
        assert_eq!(dracula_color(6), (0xFF, 0x79, 0xC6));
    }

    #[test]
    fn test_catppuccin_color_speed_1() {
        assert_eq!(catppuccin_color(1), (0x89, 0xB4, 0xFA));
    }

    #[test]
    fn test_catppuccin_color_speed_6() {
        assert_eq!(catppuccin_color(6), (0xFA, 0xB3, 0x87));
    }

    #[test]
    fn test_monokai_color_speed_1() {
        assert_eq!(monokai_color(1), (0x66, 0xD9, 0xEF));
    }

    #[test]
    fn test_monokai_color_speed_6() {
        assert_eq!(monokai_color(6), (0xAE, 0x81, 0xFF));
    }

    #[test]
    fn test_solarized_color_speed_1() {
        assert_eq!(solarized_color(1), (0x26, 0x8B, 0xD2));
    }

    #[test]
    fn test_solarized_color_speed_6() {
        assert_eq!(solarized_color(6), (0x6C, 0x71, 0xC4));
    }

    #[test]
    fn test_color_for_speed_gradient() {
        let c = color_for_speed(3, &ColorMode::Gradient);
        assert_eq!(c, Color::AnsiValue(247));
    }

    #[test]
    fn test_drop_fall_wraps() {
        let color = color_for_speed(3, &ColorMode::Gradient);
        let mut d = Drop {
            w: 10,
            h: 20,
            speed: 2,
            color,
            shape: '|',
        };
        d.fall(30, 0, 80);
        assert_eq!(d.h, 22);
        d.fall(30, 0, 80);
        assert_eq!(d.h, 24);
        d.h = 29;
        d.fall(30, 0, 80);
        assert!(d.h < 10);
    }

    #[test]
    fn test_drop_fall_wind() {
        let color = color_for_speed(3, &ColorMode::Gradient);
        let mut d = Drop {
            w: 40,
            h: 5,
            speed: 1,
            color,
            shape: '|',
        };
        d.fall(30, 3, 80);
        assert_eq!(d.w, 43);
        d.fall(30, -2, 80);
        assert_eq!(d.w, 41);
    }

    #[test]
    fn test_drop_fall_wind_wraps() {
        let color = color_for_speed(3, &ColorMode::Gradient);
        let mut d = Drop {
            w: 79,
            h: 5,
            speed: 1,
            color,
            shape: ':',
        };
        d.fall(30, 2, 80);
        assert_eq!(d.w, 1);
    }
}
