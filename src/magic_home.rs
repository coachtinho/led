use clap::Subcommand;

use std::error::Error;
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;

type Control = (u8, u8, u8);
type Function = (u8, u8);
type Rgb = (u8, u8, u8);

// Constants
// Control
const STATUS: Control = (0x81, 0x8a, 0x8b);
const ON: Control = (0x71, 0x23, 0x0f);
const OFF: Control = (0x71, 0x24, 0x0f);
// Functions
const CHAOS: Function = (49, 5);
const AMBIENT: Function = (37, 50);
const RAINBOW: Function = (37, 1);
// Colors
const RED: Rgb = (255, 0, 0);
const GREEN: Rgb = (0, 255, 0);
const BLUE: Rgb = (0, 0, 255);
const LIME: Rgb = (255, 255, 0);
const YELLOW: Rgb = (255, 110, 0);
const PINK: Rgb = (255, 0, 170);
const CYAN: Rgb = (0, 255, 255);
const PURPLE: Rgb = (170, 0, 255);
const ORANGE: Rgb = (255, 24, 0);
const WHITE: Rgb = (255, 255, 255);

const DEFAULT_PORT: &str = "5577";
const STATUS_BUFFER_SIZE: usize = 14;

enum Message {
    Control(Control),
    Function(Function),
    Color(Rgb),
}

#[derive(Subcommand, Debug)]
pub enum Actions {
    /// Get status of device
    Status,

    /// Turn on device
    On,

    /// Turn off device
    Off,

    /// Red strobe
    Chaos,

    /// Fast cycle
    Rainbow,

    /// Slow cycle
    Ambient,

    /// Red static
    Red,

    /// Green static
    Green,

    /// Blue static
    Blue,

    /// Yellow static
    Yellow,

    /// Orange static
    Orange,

    /// Lime static
    Lime,

    /// Purple static
    Purple,

    /// Pink static
    Pink,

    /// Cyan static
    Cyan,

    /// White static
    White,
}

/// Object representing the status of the LED
pub struct Status {
    power: bool,
    color: Rgb,
    mode: &'static str,
    speed: Option<u8>,
}

impl From<&[u8; 14]> for Status {
    /// Constructs Status object from byte sequence returned by device
    fn from(buffer: &[u8; STATUS_BUFFER_SIZE]) -> Self {
        // Destructure status buffer
        let [_, _, power, mode, _, speed, r, b, g, ..] = buffer;

        // Parse power
        let power = *power == 35;

        // Parse color
        let color = (*r, *g, *b);

        // Parse mode
        let (mode, speed) = match mode {
            97 => ("static", None),
            49 => ("strobe", Some(100 - speed)),
            37 => ("cycle", Some(100 - speed)),
            _ => ("unknown", None),
        };

        Status {
            power,
            color,
            mode,
            speed,
        }
    }
}

impl fmt::Display for Status {
    /// Formats status object into readable text
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();

        if self.power {
            string.push_str("Power: on\n");
        } else {
            string.push_str("Power: off\n");
        }
        string.push_str(&format!("Color: {:?}\n", self.color));
        string.push_str(&format!("Mode: {}", self.mode));
        if let Some(speed) = self.speed {
            string.push_str(&format!("\nSpeed: {}", speed));
        }

        write!(f, "{}", string)
    }
}

pub struct MagicHomeAPI(TcpStream);

impl MagicHomeAPI {
    /// Creates api from device address
    /// If no port is provided defaults to 5577
    pub fn new(address: &str, port: Option<&str>) -> Result<MagicHomeAPI, Box<dyn Error>> {
        let port = port.unwrap_or(DEFAULT_PORT);
        let address = &format!("{}:{}", address, port);
        let stream = TcpStream::connect(address)?;

        Ok(MagicHomeAPI(stream))
    }

    /// Sets color of device according to Rgb values
    #[allow(dead_code, unused_must_use)]
    pub fn set_rgb(&mut self, r: isize, g: isize, b: isize) -> Result<(), &'static str> {
        if !(0..=255).contains(&r) {
            Err("Invalid r value")
        } else if !(0..=255).contains(&g) {
            Err("Invalid g value")
        } else if !(0..=255).contains(&b) {
            Err("Invalid b value")
        } else {
            let message = Message::Color((r as u8, g as u8, b as u8));
            self.send_to_device(message);

            Ok(())
        }
    }

    /// Changes mode of device to one of the preset functions or colors or gets status of device
    pub fn perform_action(&mut self, action: &Actions) -> Result<Option<Status>, Box<dyn Error>> {
        let message = match action {
            Actions::Status => Message::Control(STATUS),
            Actions::On => Message::Control(ON),
            Actions::Off => Message::Control(OFF),
            Actions::Chaos => Message::Function(CHAOS),
            Actions::Ambient => Message::Function(AMBIENT),
            Actions::Rainbow => Message::Function(RAINBOW),
            Actions::Red => Message::Color(RED),
            Actions::Green => Message::Color(GREEN),
            Actions::Blue => Message::Color(BLUE),
            Actions::Lime => Message::Color(LIME),
            Actions::Yellow => Message::Color(YELLOW),
            Actions::Pink => Message::Color(PINK),
            Actions::Cyan => Message::Color(CYAN),
            Actions::Purple => Message::Color(PURPLE),
            Actions::Orange => Message::Color(ORANGE),
            Actions::White => Message::Color(WHITE),
        };

        self.send_to_device(message)
    }

    fn send_to_device(&mut self, message: Message) -> Result<Option<Status>, Box<dyn Error>> {
        let mut bytes = match message {
            Message::Color((r, g, b)) => vec![0x31, r, b, g, 0xff, 0x00, 0x0f],
            Message::Function((preset, speed)) => {
                // Preset functions don't turn on the device
                // by default so it mus be done manually
                self.send_to_device(Message::Control(ON))?;
                vec![0x61, preset, speed, 0x0f]
            }
            Message::Control((b1, b2, b3)) => vec![b1, b2, b3],
        };
        let checksum = MagicHomeAPI::calc_checksum(bytes.as_slice());

        bytes.push(checksum);

        self.0.write_all(bytes.as_slice())?;

        // Receive status
        if let Message::Control(STATUS) = message {
            let mut buffer: [u8; STATUS_BUFFER_SIZE] = [0; STATUS_BUFFER_SIZE];

            self.0.read_exact(&mut buffer)?;

            Ok(Some(Status::from(&buffer)))
        } else {
            Ok(None)
        }
    }

    fn calc_checksum(bytes: &[u8]) -> u8 {
        let mut checksum = 0;

        for num in bytes.iter() {
            checksum = num.wrapping_add(checksum);
        }

        checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn good_url() {
        let _a = TcpListener::bind("127.0.0.1:9999").unwrap();
        let api = MagicHomeAPI::new("127.0.0.1", Some("9999"));
        assert!(api.is_ok());
    }

    #[test]
    fn good_url_no_port() {
        let _a = TcpListener::bind("127.0.0.1:5577").unwrap();
        let api = MagicHomeAPI::new("127.0.0.1", None);
        assert!(api.is_ok());
    }

    #[test]
    fn bad_url() {
        let api = MagicHomeAPI::new("badurl", None);
        assert!(!api.is_ok());
    }

    #[test]
    fn valid_set_rgb() {
        let _a = TcpListener::bind("127.0.0.1:9997").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9997")).unwrap();
        let result = api.set_rgb(255, 1, 0);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn invalid_set_rgb_neg() {
        let _a = TcpListener::bind("127.0.0.1:9996").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9996")).unwrap();
        let result = api.set_rgb(255, -1, 0);
        assert_eq!(result, Err("Invalid g value"));
    }

    #[test]
    fn invalid_set_rgb_upper() {
        let _a = TcpListener::bind("127.0.0.1:9995").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9995")).unwrap();
        let result = api.set_rgb(255, 0, 300);
        assert_eq!(result, Err("Invalid b value"));
    }

    #[test]
    fn valid_set_mode() {
        let _a = TcpListener::bind("127.0.0.1:9994").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9994")).unwrap();
        let result = api.perform_action(&Actions::Chaos).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn calculate_checksum() {
        let bytes = vec![0x31, 0xff, 0xff, 0x00, 0xff, 0x00, 0x0f];
        let checksum = MagicHomeAPI::calc_checksum(&bytes);
        assert_eq!(checksum, 0x3d);
    }
}
