use collectd_plugin::{Plugin, PluginCapabilities, Value, ValueListBuilder};

use std::error;
use std::sync::{Arc, Mutex};

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
            data: Arc::new(Mutex::new(Data::default()))
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Data {
    /// device type
    device_type: DeviceType,
}

#[derive(Debug, PartialEq)]
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
            DeviceType::Unknown => write!(f, "Unknown")
        }
    }
}