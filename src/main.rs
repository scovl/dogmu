#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod atomic_f32;
mod config;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use enigo::{Direction, Enigo, Keyboard, Mouse};
use gilrs::{Axis, Event, EventType, Gilrs};
use single_instance::SingleInstance;

use crate::atomic_f32::*;
use crate::config::*;

struct Coordinate {
    x: AtomicF32,
    y: AtomicF32,
}

impl Coordinate {
    const fn new() -> Self {
        Self {
            x: AtomicF32::new(),
            y: AtomicF32::new(),
        }
    }

    fn reset(&self) {
        self.x.reset();
        self.y.reset();
    }
}

static IS_ALTERNATIVE_ACTIVE: AtomicBool = AtomicBool::new(false);
static LEFT_STICK_COORD: Coordinate = Coordinate::new();
static RIGHT_STICK_COORD: Coordinate = Coordinate::new();

static CONFIG: OnceLock<Config> = OnceLock::new();
static ENIGO: OnceLock<tokio::sync::Mutex<Enigo>> = OnceLock::new();
static REPEAT_KEY_ABORT_HANDLE: OnceLock<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>> =
    OnceLock::new();

fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config_path = std::env::current_exe().unwrap().with_extension("toml");
        let config_str = std::fs::read_to_string(&config_path).unwrap_or_default();
        let config =
            toml::from_str::<Config>(&config_str).expect("Unable to parse the config file");
        config.check_error().unwrap()
    })
}

fn get_enigo() -> &'static tokio::sync::Mutex<Enigo> {
    ENIGO.get_or_init(|| {
        tokio::sync::Mutex::new(
            Enigo::new(&enigo::Settings::default()).expect("Failed to initialize Enigo"),
        )
    })
}

fn get_repeat_key_abort_handle() -> &'static tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>
{
    REPEAT_KEY_ABORT_HANDLE.get_or_init(|| tokio::sync::Mutex::new(None))
}

async fn press_input(input_name: &str, is_press_down: bool) {
    if let Some(activator) = &get_config().alternative_activator {
        if input_name == activator.to_lowercase() {
            IS_ALTERNATIVE_ACTIVE.store(is_press_down, Ordering::Relaxed);
            return;
        }
    }

    if let Some(remap) = get_config().get_remap(
        input_name,
        IS_ALTERNATIVE_ACTIVE.load(Ordering::Relaxed),
    ) {
        match remap {
            Remap::Seq(seq) => {
                if is_press_down {
                    let mut enigo = get_enigo().lock().await;

                    for key in seq.iter() {
                        enigo.key(*key, Direction::Press).expect("Failed to press key");
                    }
                    for key in seq.iter().rev() {
                        enigo.key(*key, Direction::Release).expect("Failed to release key");
                    }
                }
            }
            Remap::Sync(seq) => {
                let mut enigo = get_enigo().lock().await;

                if is_press_down {
                    for key in seq.iter() {
                        enigo.key(*key, Direction::Press).expect("Failed to press key");
                    }
                } else {
                    for key in seq.iter().rev() {
                        enigo.key(*key, Direction::Release).expect("Failed to release key");
                    }
                }
            }
            Remap::Repeat(key) => {
                let mut abort_handle_lock = get_repeat_key_abort_handle().lock().await;

                if let Some(handle) = abort_handle_lock.take() {
                    handle.abort();
                }

                if is_press_down {
                    get_enigo()
                        .lock()
                        .await
                        .key(*key, Direction::Click)
                        .expect("Failed to click key");

                    let handle = tokio::spawn(async move {
                        tokio::time::sleep(get_config().key_repeat_initial_delay).await;

                        loop {
                            get_enigo()
                                .lock()
                                .await
                                .key(*key, Direction::Click)
                                .expect("Failed to click key");
                            tokio::time::sleep(get_config().key_repeat_sub_delay).await;
                        }
                    });
                    *abort_handle_lock = Some(handle);
                }
            }
            Remap::Mouse(button) => {
                get_enigo()
                    .lock()
                    .await
                    .button(
                        *button,
                        if is_press_down {
                            Direction::Press
                        } else {
                            Direction::Release
                        },
                    )
                    .expect("Failed to press/release mouse button");
            }
            Remap::Command(cmdline) => {
                if is_press_down {
                    if let Some(components) = shlex::split(cmdline) {
                        if !components.is_empty() {
                            let _ = std::process::Command::new(&components[0])
                                .args(&components[1..])
                                .spawn();
                        }
                    }
                }
            }
        }
    }
}

async fn left_stick() {
    let mouse_acceleration =
        (get_config().mouse_max_speed - get_config().mouse_initial_speed)
            / get_config().mouse_ticks_to_reach_max_speed;
    let mut curr_mouse_speed = get_config().mouse_initial_speed;

    loop {
        let x = LEFT_STICK_COORD.x.load();
        let y = LEFT_STICK_COORD.y.load();
        let distance_to_origin = (x * x + y * y).sqrt();
        let dead_zone_shrink_ratio =
            (1. - (get_config().left_stick_dead_zone) / distance_to_origin).max(0.);
        let delta_x = x * dead_zone_shrink_ratio * curr_mouse_speed;
        let delta_y = y * dead_zone_shrink_ratio * curr_mouse_speed;

        if delta_x != 0. || delta_y != 0. {
            get_enigo()
                .lock()
                .await
                .move_mouse(delta_x as i32, -delta_y as i32, enigo::Coordinate::Rel)
                .expect("Failed to move mouse");
            curr_mouse_speed = (curr_mouse_speed + mouse_acceleration).min(get_config().mouse_max_speed);
        } else {
            curr_mouse_speed = get_config().mouse_initial_speed;
        }

        tokio::time::sleep(get_config().left_stick_poll_interval).await;
    }
}

async fn right_stick() {
    const TRIGGER_ANGLES: [f32; 4] = [
        1. * std::f32::consts::FRAC_PI_8,
        3. * std::f32::consts::FRAC_PI_8,
        5. * std::f32::consts::FRAC_PI_8,
        7. * std::f32::consts::FRAC_PI_8,
    ];
    let mut pressed_input_name = None;

    loop {
        let x = RIGHT_STICK_COORD.x.load();
        let y = RIGHT_STICK_COORD.y.load();
        let distance_to_origin = (x * x + y * y).sqrt();

        if distance_to_origin <= get_config().right_stick_dead_zone {
            if let Some(input_name) = pressed_input_name.take() {
                press_input(input_name, false).await;
            }
        } else if distance_to_origin >= get_config().right_stick_trigger_zone && pressed_input_name.is_none() {
            let stick_angle = y.atan2(x);

            pressed_input_name = if stick_angle >= TRIGGER_ANGLES[1] && stick_angle <= TRIGGER_ANGLES[2] {
                Some("right_stick_up")
            } else if stick_angle >= -TRIGGER_ANGLES[2] && stick_angle <= -TRIGGER_ANGLES[1] {
                Some("right_stick_down")
            } else if stick_angle >= TRIGGER_ANGLES[3] || stick_angle <= -TRIGGER_ANGLES[3] {
                Some("right_stick_left")
            } else if stick_angle >= -TRIGGER_ANGLES[0] && stick_angle <= TRIGGER_ANGLES[0] {
                Some("right_stick_right")
            } else {
                None
            };

            if let Some(input_name) = pressed_input_name {
                press_input(input_name, true).await;
            }
        }

        tokio::time::sleep(get_config().right_stick_poll_interval).await;
    }
}

fn get_button_input_name(button: gilrs::Button) -> Option<&'static str> {
    match button {
        gilrs::Button::North => Some("north"),
        gilrs::Button::South => Some("south"),
        gilrs::Button::West => Some("west"),
        gilrs::Button::East => Some("east"),
        gilrs::Button::LeftTrigger => Some("left_bumper"),
        gilrs::Button::RightTrigger => Some("right_bumper"),
        gilrs::Button::LeftTrigger2 => Some("left_trigger"),
        gilrs::Button::RightTrigger2 => Some("right_trigger"),
        gilrs::Button::Select => Some("select"),
        gilrs::Button::Start => Some("start"),
        gilrs::Button::Mode => Some("mode"),
        gilrs::Button::LeftThumb => Some("left_thumb"),
        gilrs::Button::RightThumb => Some("right_thumb"),
        gilrs::Button::DPadUp => Some("dpad_up"),
        gilrs::Button::DPadDown => Some("dpad_down"),
        gilrs::Button::DPadLeft => Some("dpad_left"),
        gilrs::Button::DPadRight => Some("dpad_right"),
        _ => None,
    }
}

#[tokio::main(worker_threads = 3)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = SingleInstance::new(
        &std::env::current_exe()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy(),
    )?;
    if !instance.is_single() {
        return Ok(());
    }

    // Ignore terminal process state
    #[cfg(target_os = "linux")]
    unsafe {
        use nix::sys::signal::*;
        sigaction(
            Signal::SIGCHLD,
            &SigAction::new(
                SigHandler::SigDfl,
                SaFlags::SA_NOCLDWAIT,
                SigSet::empty(),
            ),
        )?;
    }

    tokio::spawn(left_stick());
    tokio::spawn(right_stick());

    let mut gilrs = Gilrs::new()?;
    loop {
        if let Some(Event { event, .. }) = gilrs.next_event_blocking(None) {
            match event {
                EventType::Disconnected => {
                    IS_ALTERNATIVE_ACTIVE.store(false, Ordering::Relaxed);
                    LEFT_STICK_COORD.reset();
                    RIGHT_STICK_COORD.reset();
                }
                EventType::ButtonPressed(button, ..) => {
                    if let Some(input_name) = get_button_input_name(button) {
                        press_input(input_name, true).await;
                    }
                }
                EventType::ButtonReleased(button, ..) => {
                    if let Some(input_name) = get_button_input_name(button) {
                        press_input(input_name, false).await;
                    }
                }
                EventType::AxisChanged(axis, value, ..) => match axis {
                    Axis::LeftStickX => LEFT_STICK_COORD.x.store(value),
                    Axis::LeftStickY => LEFT_STICK_COORD.y.store(value),
                    Axis::RightStickX => RIGHT_STICK_COORD.x.store(value),
                    Axis::RightStickY => RIGHT_STICK_COORD.y.store(value),
                    _ => (),
                },
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_baseline() {
        press_input("", true).await;
        // Note: the left_stick and right_stick loops are infinity;
    }
}
