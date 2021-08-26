use chrono::{Datelike, FixedOffset, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use zellij_tile::prelude::*;

const STATE_SAVING_PATH: &str = "/data/pomo.json";
const WORKING_INTERVAL: Duration = Duration::from_secs(1500); // 25 min
const BREAKING_INTERVAL: Duration = Duration::from_secs(300); // 5 min
const NAPPING_INTERVAL: Duration = Duration::from_secs(900); // 15 min

#[derive(Serialize, Deserialize, Clone, Copy)]
enum Status {
    Working(usize, Duration),
    Resting(usize, Duration),
    Napping(Duration),
}

impl Default for Status {
    fn default() -> Self {
        Status::Working(0, WORKING_INTERVAL)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Working(i, d) => write!(
                f,
                "Working(round {}): remaining {:02}:{:02}",
                i + 1,
                d.as_secs() / 60,
                d.as_secs() % 60
            ),
            Status::Resting(i, d) => write!(
                f,
                "Resting(round {}): remaining {:02}:{:02}",
                i + 1,
                d.as_secs() / 60,
                d.as_secs() % 60
            ),
            Status::Napping(d) => write!(
                f,
                "Napping: remaining {:02}:{:02}",
                d.as_secs() / 60,
                d.as_secs() % 60
            ),
        }
    }
}

impl Status {
    fn elapsed(self, d: Duration) -> Self {
        match self {
            Status::Working(i, remain) => {
                if let Some(remain) = remain.checked_sub(d) {
                    Status::Working(i, remain)
                } else {
                    exec_cmd(&vec!["notify-send", "pomodoro", "Time to take a break"]);
                    Status::Resting(i, BREAKING_INTERVAL)
                }
            }
            Status::Resting(i, remain) => {
                if let Some(remain) = remain.checked_sub(d) {
                    Status::Resting(i, remain)
                } else {
                    if i + 1 == 4 {
                        exec_cmd(&vec!["notify-send", "pomodoro", "Time to take some nap"]);
                        Status::Napping(NAPPING_INTERVAL)
                    } else {
                        exec_cmd(&vec!["notify-send", "pomodoro", "Time to start working"]);
                        Status::Working(i + 1, WORKING_INTERVAL)
                    }
                }
            }
            Status::Napping(remain) => {
                if let Some(remain) = remain.checked_sub(d) {
                    Status::Napping(remain)
                } else {
                    exec_cmd(&vec!["notify-send", "pomodoro", "Time to start working"]);
                    Status::Working(0, WORKING_INTERVAL)
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Pomo {
    paused: bool,
    status: Status,
}

impl Pomo {
    fn new() -> Self {
        Pomo::default()
    }

    fn elapsed(&mut self, dur: Duration) {
        if self.paused {
            return;
        }
        self.status = self.status.elapsed(dur);
    }

    fn toggle_pause(&mut self) {
        self.paused ^= true;
    }

    fn shortcuts(&self) -> String {
        format!(
            "Tip: <space> => {pause_or_resume}, <r> => reset",
            pause_or_resume = if self.paused { "resume" } else { "pause" }
        )
    }
}

impl fmt::Display for Pomo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.status,
            if self.paused { " [paused]" } else { "" },
        )
    }
}

#[derive(Default)]
struct State {
    active: bool,
    pomo: Pomo,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self) {
        subscribe(&[EventType::KeyPress, EventType::Timer, EventType::Visible]);
    }

    fn update(&mut self, event: Event) {
        match event {
            Event::KeyPress(Key::Char('r')) => self.pomo = Pomo::new(),
            Event::KeyPress(Key::Char(' ')) => self.pomo.toggle_pause(),
            Event::Timer(t) => {
                if self.active {
                    self.pomo.elapsed(Duration::from_secs_f64(t));
                    set_timeout(1.0);
                }
            }
            Event::Visible(true) => {
                self.active = true;
                set_timeout(1.0);

                match fs::File::open(STATE_SAVING_PATH).and_then(|f| {
                    serde_json::from_reader(f).map_err(|e| Error::new(ErrorKind::Other, e))
                }) {
                    Ok(pomo) => self.pomo = pomo,
                    Err(_) => self.pomo = Pomo::new(),
                }
            }
            Event::Visible(false) => {
                self.active = false;
                fs::write(STATE_SAVING_PATH, serde_json::to_vec(&self.pomo).unwrap()).unwrap();
            }
            _ => (),
        }
    }

    fn render(&mut self, rows: usize, _cols: usize) {
        let china_timezone = FixedOffset::east(8 * 3600);
        let now = Utc::now().with_timezone(&china_timezone);
        println!(
            "{pomo} | {hour:02}:{minute:02} {year}-{month:02}-{day:02} {weekday}",
            pomo = self.pomo,
            hour = now.hour(),
            minute = now.minute(),
            year = now.year(),
            month = now.month(),
            day = now.day(),
            weekday = now.weekday(),
        );
        if rows > 1 {
            println!("{}", self.pomo.shortcuts());
        }
    }
}
