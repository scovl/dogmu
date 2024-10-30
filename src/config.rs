use std::collections::HashMap;
use std::time::Duration;

use duration_str::deserialize_duration;

/// Represents different types of input remappings.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Remap {
    /// A sequence of keys to be pressed and released in order.
    Seq(Vec<enigo::Key>),
    /// A set of keys to be pressed and released simultaneously.
    Sync(Vec<enigo::Key>),
    /// A key to be repeated while the input is active.
    Repeat(enigo::Key),
    /// A mouse button action.
    Mouse(enigo::Button),
    /// A command-line instruction to execute.
    Command(String),
}

/// Configuration settings for input remapping and behavior.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    /// Initial delay before key repeat starts.
    #[serde(
        deserialize_with = "deserialize_duration",
        default = "Config::default_key_repeat_initial_delay"
    )]
    pub key_repeat_initial_delay: Duration,
    /// Delay between subsequent key repeats.
    #[serde(
        deserialize_with = "deserialize_duration",
        default = "Config::default_key_repeat_sub_delay"
    )]
    pub key_repeat_sub_delay: Duration,

    /// Polling interval for the left stick.
    #[serde(
        deserialize_with = "deserialize_duration",
        default = "Config::default_left_stick_poll_interval"
    )]
    pub left_stick_poll_interval: Duration,
    /// Dead zone threshold for the left stick.
    #[serde(default = "Config::default_left_stick_dead_zone")]
    pub left_stick_dead_zone: f32,

    /// Initial speed for mouse movement.
    #[serde(default = "Config::default_mouse_initial_speed")]
    pub mouse_initial_speed: f32,
    /// Maximum speed for mouse movement.
    #[serde(default = "Config::default_mouse_max_speed")]
    pub mouse_max_speed: f32,
    /// Number of ticks to reach maximum mouse speed.
    #[serde(default = "Config::default_mouse_ticks_to_reach_max_speed")]
    pub mouse_ticks_to_reach_max_speed: f32,

    /// Polling interval for the right stick.
    #[serde(
        deserialize_with = "deserialize_duration",
        default = "Config::default_right_stick_poll_interval"
    )]
    pub right_stick_poll_interval: Duration,
    /// Trigger zone threshold for the right stick.
    #[serde(default = "Config::default_right_stick_trigger_zone")]
    pub right_stick_trigger_zone: f32,
    /// Dead zone threshold for the right stick.
    #[serde(default = "Config::default_right_stick_dead_zone")]
    pub right_stick_dead_zone: f32,

    /// Optional activator for the alternative remap set.
    pub alternative_activator: Option<String>,

    /// Main remap configuration.
    pub main: HashMap<String, Remap>,
    /// Alternative remap configuration.
    pub alt: HashMap<String, Remap>,
}

impl Config {
    /// Validates the configuration and returns an error if invalid.
    pub fn check_error(self) -> Result<Self, &'static str> {
        if self.left_stick_dead_zone <= 0.0
            || self.right_stick_trigger_zone <= 0.0
            || self.right_stick_dead_zone <= 0.0
        {
            return Err("Negative zone size");
        }

        if self.right_stick_trigger_zone < self.right_stick_dead_zone {
            return Err("Trigger zone smaller than dead zone");
        }

        if let Some(activator) = &self.alternative_activator {
            if self.main.contains_key(activator) {
                return Err("Activator for alternative set is remapped");
            }
        }

        Ok(self)
    }

    /// Retrieves the remap for a given input, considering the active remap set.
    ///
    /// # Arguments
    ///
    /// * `input` - The input name to remap.
    /// * `is_alternative` - Whether to use the alternative remap set.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `Remap` if found.
    pub fn get_remap(&self, input: &str, is_alternative: bool) -> Option<&Remap> {
        if is_alternative {
            self.alt.get(input)
        } else {
            self.main.get(input)
        }
    }

    // Default values for configuration settings.

    fn default_key_repeat_initial_delay() -> Duration {
        Duration::from_millis(400)
    }

    fn default_key_repeat_sub_delay() -> Duration {
        Duration::from_millis(40)
    }

    fn default_left_stick_poll_interval() -> Duration {
        Duration::from_millis(10)
    }

    fn default_left_stick_dead_zone() -> f32 {
        0.05
    }

    fn default_mouse_initial_speed() -> f32 {
        10.0
    }

    fn default_mouse_max_speed() -> f32 {
        20.0
    }

    fn default_mouse_ticks_to_reach_max_speed() -> f32 {
        30.0
    }

    fn default_right_stick_poll_interval() -> Duration {
        Duration::from_millis(50)
    }

    fn default_right_stick_trigger_zone() -> f32 {
        0.3
    }

    fn default_right_stick_dead_zone() -> f32 {
        0.1
    }
}
