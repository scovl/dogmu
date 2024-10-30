# Controller to Keyboard and Mouse Remapping Tool

A utility to remap controller inputs to keyboard and mouse actions.

## Features

- **Left Stick Mouse Control:** Map the controller's left stick to mouse movements with configurable acceleration.
- **Right Stick Keyboard Inputs:** Map the controller's right stick to keyboard inputs, simulating directional keys or custom commands.
- **Key Repeat Simulation:** Hold a controller button to simulate key repeats, ensuring the initial input is always registered.
- **Sequential Key Inputs:** Press a controller button to trigger a sequence of key presses and releases in a specified order.
- **Executable Commands Execution:** Run executable commands by pressing a controller button.
- **Alternative Mapping Sets:** Toggle between two sets of mappings using an "activator" button.
- **Cross-Platform Support:** Compatible with both Windows and Linux operating systems.

## Configuration

The application looks for a configuration file named `<executable_name>.toml` in the same directory as the executable. This TOML file defines how controller inputs are remapped to keyboard and mouse actions.

### Top-Level Configuration Options

- `key_repeat_initial_delay` *(Duration String)*: Delay before key repeats start after the initial input. Supports various human-readable formats like `'400ms'`. Uses [duration_str](https://docs.rs/duration-str/latest/duration_str/).
- `key_repeat_sub_delay` *(Duration String)*: Delay between subsequent key repeats.
- `left_stick_poll_interval` *(Duration String)*: Interval at which the left stick is polled for movement.
- `left_stick_dead_zone` *(Decimal)*: Threshold for the left stick's dead zone. Movements within this zone are ignored. Value between `0` and `1`.
- `mouse_initial_speed` *(Decimal)*: Initial speed for mouse movement when using the left stick.
- `mouse_max_speed` *(Decimal)*: Maximum speed for mouse movement.
- `mouse_ticks_to_reach_max_speed` *(Decimal)*: Number of polling intervals it takes to reach maximum mouse speed.
- `right_stick_poll_interval` *(Duration String)*: Interval at which the right stick is polled.
- `right_stick_trigger_zone` *(Decimal)*: Threshold for the right stick's trigger zone. Movements within this zone are considered active. Value between `0` and `1`.
- `right_stick_dead_zone` *(Decimal)*: Threshold for the right stick's dead zone. Movements within this zone are ignored.
- `alternative_activator` *(String)*: Name of the controller button that, when held down, switches to the alternative mapping set.

### Mapping Sets

There are two predefined mapping sets: `main` and `alt`. By default, the `main` set is active. Holding down the `alternative_activator` button switches to the `alt` set.

- **Controller Input Names:** Based on [gilrs' naming convention](https://docs.rs/gilrs/latest/gilrs/ev/enum.Button.html#variants), converted to snake_case. Examples include `left_bumper`, `right_trigger`, `dpad_up`.
- **Right Stick Directions:** To map the right stick as a 4-way input, use `right_stick_up`, `right_stick_down`, `right_stick_left`, and `right_stick_right`.
- **Keyboard Output Names:** Based on [enigo's naming convention](https://docs.rs/enigo/latest/enigo/enum.Key.html#variants). Examples include `Control`, `Shift`, `PageUp`.
  
### Mapping Values

Each mapping is defined as a key-value pair within the mapping set, where the key is the controller input name, and the value is a table specifying the remap action.

Supported mapping keys:

- `seq`: A sequence of keyboard keys to be pressed and released in order when the controller button is pressed down and released. Keys are pressed down in sequence and released in reverse order.
  ```toml
  east = { seq = ['Control', 'C'] }
  ```
- `sync`: A list of keyboard keys to be pressed down when the controller button is pressed and released when the button is released.
  ```toml
  right_bumper = { sync = ['Shift'] }
  ```
- `repeat`: A single keyboard key to be repeatedly pressed while the controller button is held down.
  ```toml
  south = { repeat = 'Return' }
  ```
- `mouse`: Simulate a mouse button press. Accepts `'Left'`, `'Right'`, or `'Middle'`.
  ```toml
  left_trigger = { mouse = 'Right' }
  ```
- `command`: Execute a system command or run an executable.
  ```toml
  start = { command = '/path/to/script.sh' }
  ```

### Example Configuration

```toml
key_repeat_initial_delay = '400ms'
key_repeat_sub_delay = '40ms'
left_stick_poll_interval = '10ms'
left_stick_dead_zone = 0.05
mouse_initial_speed = 10.0
mouse_max_speed = 20.0
mouse_ticks_to_reach_max_speed = 30.0
right_stick_poll_interval = '50ms'
right_stick_trigger_zone = 0.3
right_stick_dead_zone = 0.1
alternative_activator = 'select'

[main]
north = { repeat = 'PageUp' }
south = { repeat = 'Return' }
west = { repeat = 'PageDown' }
east = { repeat = 'Space' }
left_bumper = { mouse = 'Left' }
right_bumper = { sync = ['Control'] }
left_trigger = { mouse = 'Right' }
right_trigger = { mouse = 'Middle' }
start = { sync = ['Shift'] }
left_thumb = { seq = ['Control', 'Meta', 'O'] }
right_thumb = { seq = ['Control', 'W'] }
dpad_up = { repeat = 'UpArrow' }
dpad_down = { repeat = 'DownArrow' }
dpad_left = { repeat = 'LeftArrow' }
dpad_right = { repeat = 'RightArrow' }
right_stick_up = { seq = ['F5'] }
right_stick_down = { seq = ['Control', 'C'] }
right_stick_left = { seq = ['Control', 'V'] }
right_stick_right = { seq = ['Control', 'Alt', 'Tab'] }

[alt]
north = { seq = ['Home'] }
south = { seq = ['End'] }
west = { repeat = 'Backspace' }
east = { repeat = 'Delete' }
right_stick_up = { seq = ['Escape'] }
right_stick_down = { seq = ['Meta', 'D'] }
right_stick_left = { seq = ['Control', 'Alt', { Unicode = '1' }] }
right_stick_right = { seq = ['Meta', 'X'] }
```

## Usage

1. **Create Configuration File:** Place a TOML file named after the executable (e.g., `controller_remap.toml`) in the same directory. Use the configuration format described above.

2. **Run the Application:** Execute the compiled binary. The application will read the configuration file and start remapping controller inputs accordingly.

3. **Switch Between Mapping Sets:** Hold down the `alternative_activator` button (e.g., `select`) on your controller to switch to the alternative mapping set.

## Dependencies

- **Rust Toolchain:** Ensure you have the latest stable Rust toolchain installed.
- **Libraries:**
  - [`enigo`](https://crates.io/crates/enigo): For simulating keyboard and mouse inputs.
  - [`gilrs`](https://crates.io/crates/gilrs): For handling gamepad inputs.
  - [`tokio`](https://crates.io/crates/tokio): For asynchronous runtime support.
  - [`serde`](https://crates.io/crates/serde) and [`toml`](https://crates.io/crates/toml): For configuration parsing.
  - [`duration_str`](https://crates.io/crates/duration_str): For parsing human-readable duration strings.

## Building from Source

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/yourusername/controller-remap.git
   cd controller-remap
   ```

2. **Build the Application:**

   ```bash
   cargo build --release
   ```

3. **Run the Application:**

   ```bash
   ./target/release/controller-remap
   ```
