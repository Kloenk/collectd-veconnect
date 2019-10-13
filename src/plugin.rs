use collectd_plugin::{Plugin, PluginCapabilities, Value, ValueListBuilder};

use std::error;
use std::sync::{Arc, Mutex};

use log::{info, warn};

#[derive(Debug, Default)]
pub(crate) struct VePlugin {
    pub(crate) name: String,
    pub(crate) data: Arc<Mutex<Data>>,
}

impl Plugin for VePlugin {
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::READ
    }

    fn read_values(&self) -> Result<(), Box<dyn error::Error>> {
        let values = vec![Value::Gauge(15.0), Value::Gauge(10.0), Value::Gauge(12.0)];

        ValueListBuilder::new("veconnect", "load")
            .values(&values)
            .plugin_instance(self.name.as_str())
            .submit()?;

        Ok(())
    }
}

use std::convert::From;

impl From<super::VePhoenixConfig> for VePlugin {
    fn from(config: super::VePhoenixConfig) -> Self {
        Self {
            name: config.name.unwrap(),
            data: Arc::new(Mutex::new(Data::new_phoenix())),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Data {
    /// device type
    device_type: DeviceType,
}

impl Data {
    pub(crate) fn new_phoenix() -> Self {
        let mut data: Self = Default::default();
        data.device_type = DeviceType::Phoenix(0,0,0,None);
        data
    }

    pub(crate) fn parse(&mut self, cmd: &str) -> Result<(), ()>{
        let cmd = cmd.trim().trim_matches(':');
        if !cmd.starts_with('7') {
            warn!("got a not request methode: {}", cmd);
            return Err(());
        }
        info!("parsing {}", cmd);
        let t: Vec<char> = cmd.chars().collect();

        let addr = parse_u16(&t[1..5]).unwrap_or(0); // TODO: add len checking
        // TODO: add checksum checking
        match addr {
            0x0100 => { // Product Id
                if t.len() != 17 {
                    warn!("product id is not 17 long: {}", t.len());
                    return Err(());
                }
                let id = parse_u16(&t[10..15]).unwrap_or(0);
                warn!("product id not implemented: {}", cmd)
            },
            0x0101 => { // Hardware revision
                warn!("hardware revision not implemented: {}", cmd);
            },
            0x0102 => {
                warn!("software version not implemented: {}", cmd);
            }
            0x010A => {
                warn!("serial number not implemented: {}", cmd);
            }

            _ => {
                warn!("unknonw command: {} - ({})", addr, cmd);
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub(crate) enum DeviceType {
    /// Phoenix series
    ///
    /// # Values
    /// 1. Voltage
    /// 2. VA
    /// 3. AC Voltage
    /// 4. resolution
    Phoenix(u8, u16, u8, Option<String>),

    Unknown,
}

impl Default for DeviceType {
    fn default() -> DeviceType {
        DeviceType::Unknown
    }
}

use std::fmt;

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceType::Phoenix(v, w, av, r) => {
                if let Some(r) = r {
                    write!(f, "Phoenix {}V {}VA {}Vac {}", v, w, av, r)
                } else {
                    write!(f, "Phoenix {}V {}VA {}Vac", v, w, av)
                }
            }
            DeviceType::Unknown => write!(f, "Unknown"),
        }
    }
}




/// parse 4 chars a little endian to u16
/// 
/// # Example
/// ```
/// let p = parse_u16(&['F', '0', 'E', 'D']).unwrap();
/// assert_eq!(p, 0xedf0);
/// ```
pub(crate) fn parse_u16(input: &[char]) -> Result<u16, String> {    // TODO: implement error type
    if input.len() != 4 {
        return Err("invalid lenght".to_string());
    }
    let addr_low: String = input[0..2].iter().collect(); // FIX detection of not available addresses
    let addr_low = i64::from_str_radix(&addr_low, 16).unwrap_or(0); // FIXME: error return
    let addr_high: String = input[2..4].iter().collect();
    let addr_high = i64::from_str_radix(&addr_high, 16).unwrap_or(0);   //FIXME: error return
    let addr = addr_low + (addr_high * 256);
    let addr = addr as u16;
    Ok(addr)
}

#[cfg(test)]
mod test {
    #[test]
    fn parse_u16() {
        let p = super::parse_u16(&['F', '0', 'E', 'D']).unwrap();
        assert_eq!(p, 0xedf0);
    }
}