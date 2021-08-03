use std::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;

pub struct MagicHomeAPI(TcpStream);

enum Mode {
    Color(u8, u8, u8),
    Function(u8, u8),
}

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
    #[allow(dead_code)]
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

    /// Converts string to RGB values
    fn string_to_rgb(color: &str) -> Result<Mode, &'static str> {
        match color {
            "red" => Ok(Mode::Color(255, 0, 0)),
            "green" => Ok(Mode::Color(0, 255, 0)),
            "blue" => Ok(Mode::Color(0, 0, 255)),
            "lime" => Ok(Mode::Color(255, 255, 0)),
            "yellow" => Ok(Mode::Color(255, 110, 0)),
            "pink" => Ok(Mode::Color(255, 0, 170)),
            "cyan" => Ok(Mode::Color(0, 255, 255)),
            "purple" => Ok(Mode::Color(170, 0, 255)),
            "orange" => Ok(Mode::Color(255, 24, 0)),
            "white" => Ok(Mode::Color(255, 255, 255)),
            _ => Err("Invalid color"),
        }
    }

    /// Changes mode of device to one of the preset functions or colors
    pub fn set_mode(&mut self, mode: &str) -> Result<(), &'static str> {
        let mode = match mode {
            "chaos" => Mode::Function(49, 5),
            "ambient" => Mode::Function(37, 50),
            "rainbow" => Mode::Function(37, 1),
            _ => MagicHomeAPI::string_to_rgb(mode)?,
        };
        self.send_to_device(mode);

        Ok(())
    }

    fn send_to_device(&mut self, mode: Mode) {
        let mut message = match mode {
            Mode::Color(r, g, b) => vec![0x31, r, b, g, 0xff, 0x00, 0x0f],
            Mode::Function(preset, speed) => {
                // Preset functions don't turn on the device
                // by default so it mus be done manually
                self.turn_on();
                vec![0x61, preset, speed, 0x0f]
            }
        };
        let checksum = MagicHomeAPI::calc_checksum(message.as_slice());

        message.push(checksum);

        self.0
            .write_all(message.as_slice())
            .expect("Failed writing to socket");
    }

    fn calc_checksum(bytes: &[u8]) -> u8 {
        let mut checksum = 0;

        for num in bytes.iter() {
            checksum = num.wrapping_add(checksum);
        }

        checksum
    }

    pub fn turn_on(&mut self) {
        self.0
            .write_all(&[0x71, 0x23, 0x0f, 0xa3])
            .expect("Failed writing to socket");
    }

    pub fn turn_off(&mut self) {
        self.0
            .write_all(&[0x71, 0x24, 0x0f, 0xa4])
            .expect("Failed writing to socket");
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
    fn valid_string_to_rgb() {
        if let Ok(Mode::Color(r, g, b)) = MagicHomeAPI::string_to_rgb("yellow") {
            assert_eq!(r, 255);
            assert_eq!(g, 110);
            assert_eq!(b, 0);
        } else {
            panic!("Returned Err");
        }
    }

    #[test]
    fn invalid_string_to_rgb() {
        if let Err(error) = MagicHomeAPI::string_to_rgb("black") {
            assert_eq!(error, "Invalid color");
        } else {
            panic!("Did not return Err");
        }
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
        let result = api.set_mode("chaos");
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn invalid_set_mode() {
        let _a = TcpListener::bind("127.0.0.1:9994").unwrap();
        let mut api = MagicHomeAPI::new("127.0.0.1", Some("9994")).unwrap();
        let result = api.set_mode("reed");
        assert_eq!(result, Err("Invalid color"));
    }

    #[test]
    fn calculate_checksum() {
        let bytes = vec![0x31, 0xff, 0xff, 0x00, 0xff, 0x00, 0x0f];
        let checksum = MagicHomeAPI::calc_checksum(&bytes);
        assert_eq!(checksum, 0x3d);
    }
}
