use clap::Subcommand;

use std::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;

enum Mode {
    Color(u8, u8, u8),
    Function(u8, u8),
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

pub struct Status<'a> {
    pub power: bool,
    pub color: (u8, u8, u8),
    pub mode: &'a str,
    pub speed: Option<u8>,
}

impl From<&[u8; 14]> for Status<'_> {
    fn from(buffer: &[u8; 14]) -> Self {
        // Parse power
        let power = buffer[2] == 35;

        // Parse color
        let color = (buffer[6], buffer[8], buffer[7]);

        // Parse mode
        let (mode, speed) = match buffer[3] {
            97 => ("static", None),
            49 => ("strobe", Some(100 - buffer[5])),
            37 => ("cycle", Some(100 - buffer[5])),
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

pub struct MagicHomeAPI(TcpStream);

impl MagicHomeAPI {
    /// Creates api from device address
    /// If no port is provided defaults to 5577
    pub fn new(address: &str, port: Option<&str>) -> Result<MagicHomeAPI, Box<dyn Error>> {
        let port = port.unwrap_or("5577");
        let address = &format!("{}:{}", address, port);
        let stream = TcpStream::connect(address)?;

        Ok(MagicHomeAPI(stream))
    }

    /// Sets color of device according to RGB values
    #[allow(dead_code, unused_must_use)]
    pub fn set_rgb(&mut self, r: isize, g: isize, b: isize) -> Result<(), &'static str> {
        if !(0..=255).contains(&r) {
            Err("Invalid r value")
        } else if !(0..=255).contains(&g) {
            Err("Invalid g value")
        } else if !(0..=255).contains(&b) {
            Err("Invalid b value")
        } else {
            let mode = Mode::Color(r as u8, g as u8, b as u8);
            self.send_to_device(mode);

            Ok(())
        }
    }

    /// Changes mode of device to one of the preset functions or colors or gets status of device
    pub fn perform_action(&mut self, action: &Actions) -> Result<Option<Status>, Box<dyn Error>> {
        match action {
            Actions::Status => Ok(Some(self.get_status()?)),
            Actions::On => {
                self.turn_on()?;
                Ok(None)
            }
            Actions::Off => {
                self.turn_off()?;
                Ok(None)
            }
            Actions::Chaos => {
                self.send_to_device(Mode::Function(49, 5))?;
                Ok(None)
            }
            Actions::Ambient => {
                self.send_to_device(Mode::Function(37, 50))?;
                Ok(None)
            }
            Actions::Rainbow => {
                self.send_to_device(Mode::Function(37, 1))?;
                Ok(None)
            }
            Actions::Red => {
                self.send_to_device(Mode::Color(255, 0, 0))?;
                Ok(None)
            }
            Actions::Green => {
                self.send_to_device(Mode::Color(0, 255, 0))?;
                Ok(None)
            }
            Actions::Blue => {
                self.send_to_device(Mode::Color(0, 0, 255))?;
                Ok(None)
            }
            Actions::Lime => {
                self.send_to_device(Mode::Color(255, 255, 0))?;
                Ok(None)
            }
            Actions::Yellow => {
                self.send_to_device(Mode::Color(255, 110, 0))?;
                Ok(None)
            }
            Actions::Pink => {
                self.send_to_device(Mode::Color(255, 0, 170))?;
                Ok(None)
            }
            Actions::Cyan => {
                self.send_to_device(Mode::Color(0, 255, 255))?;
                Ok(None)
            }
            Actions::Purple => {
                self.send_to_device(Mode::Color(170, 0, 255))?;
                Ok(None)
            }
            Actions::Orange => {
                self.send_to_device(Mode::Color(255, 24, 0))?;
                Ok(None)
            }
            Actions::White => {
                self.send_to_device(Mode::Color(255, 255, 255))?;
                Ok(None)
            }
        }
    }

    fn send_to_device(&mut self, mode: Mode) -> Result<(), Box<dyn Error>> {
        let mut message = match mode {
            Mode::Color(r, g, b) => vec![0x31, r, b, g, 0xff, 0x00, 0x0f],
            Mode::Function(preset, speed) => {
                // Preset functions don't turn on the device
                // by default so it mus be done manually
                self.turn_on()?;
                vec![0x61, preset, speed, 0x0f]
            }
        };
        let checksum = MagicHomeAPI::calc_checksum(message.as_slice());

        message.push(checksum);

        self.0.write_all(message.as_slice())?;

        Ok(())
    }

    fn calc_checksum(bytes: &[u8]) -> u8 {
        let mut checksum = 0;

        for num in bytes.iter() {
            checksum = num.wrapping_add(checksum);
        }

        checksum
    }

    pub fn turn_on(&mut self) -> Result<(), Box<dyn Error>> {
        self.0.write_all(&[0x71, 0x23, 0x0f, 0xa3])?;

        Ok(())
    }

    pub fn turn_off(&mut self) -> Result<(), Box<dyn Error>> {
        self.0.write_all(&[0x71, 0x24, 0x0f, 0xa4])?;

        Ok(())
    }

    pub fn get_status(&mut self) -> Result<Status, Box<dyn Error>> {
        let mut buffer: [u8; 14] = [0; 14];

        self.0.write_all(&[0x81, 0x8a, 0x8b, 0x96])?;

        self.0.read_exact(&mut buffer)?;

        Ok(Status::from(&buffer))
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
        let _a = TcpListener::bind("127.0.0.1:9998").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9998")).unwrap();
        let result = api.set_rgb(255, 1, 0);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn invalid_set_rgb_neg() {
        let _a = TcpListener::bind("127.0.0.1:9997").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9997")).unwrap();
        let result = api.set_rgb(255, -1, 0);
        assert_eq!(result, Err("Invalid g value"));
    }

    #[test]
    fn invalid_set_rgb_upper() {
        let _a = TcpListener::bind("127.0.0.1:9996").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9996")).unwrap();
        let result = api.set_rgb(255, 0, 300);
        assert_eq!(result, Err("Invalid b value"));
    }

    #[test]
    fn valid_set_mode() {
        let _a = TcpListener::bind("127.0.0.1:9995").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9995")).unwrap();
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
