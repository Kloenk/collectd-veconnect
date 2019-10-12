#![macro_use]
extern crate log;

use collectd_plugin::{
    collectd_plugin, CollectdLoggerBuilder, ConfigItem, Plugin, PluginManager, PluginRegistration,
};

use log::{warn, info, LevelFilter};

//use failure::Error;
use serde::Deserialize;

use std::collections::HashMap;
use std::error;
use std::sync::{Arc, Mutex};

pub(crate) mod plugin;
use plugin::VePlugin;

#[derive(Deserialize, Debug, PartialEq)]
//#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
struct VeConfig {
    #[serde(rename = "Port")]
    port: Option<u16>,

    #[serde(rename = "Phoenix")]
    phoenix: Option<HashMap<String, VePhoenixConfig>>,
}

#[derive(Deserialize, Debug, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct VePhoenixConfig {
    pub(crate) port: Option<String>,
    pub(crate) name: Option<String>,
}

#[derive(Debug, PartialEq, Default)]
struct VePhoenixPlugin {
    config: VePhoenixConfig,
}


use plugin::Data;
struct Manager;

impl PluginManager for Manager {
    fn name() -> &'static str {
        "veconnect"
    }

    fn plugins(
        config: Option<&[ConfigItem<'_>]>,
    ) -> Result<PluginRegistration, Box<dyn error::Error>> {
        // hook rust logging into collectd's logging
        CollectdLoggerBuilder::new()
            .prefix_plugin::<Self>()
            .filter_level(LevelFilter::Info)
            .try_init()
            .expect("logger init failed");
        // Deserialize the collectd configuration into our configuration struct
        let mut config: VeConfig =
            collectd_plugin::de::from_collectd(config.unwrap_or_else(Default::default))?;

        warn!("remove hard coded devices");
        let mut phoenix = HashMap::with_capacity(1);
        phoenix.insert(
            "s0".to_string(),
            VePhoenixConfig {
                port: Some("s0".to_string()),
                name: Some("Phoenix 1".to_string()),
            },
        );
        phoenix.insert(
            "s1".to_string(),
            VePhoenixConfig {
                port: Some("s1".to_string()),
                name: Some("s1".to_string()),
            },
        );
        config.phoenix = Some(phoenix);

        println!("conf: {:?}", config);

        let mut plugins: Vec<(String, Box<dyn Plugin>)> = Vec::new();
        let mut data: HashMap<String, Arc<Mutex<Data>>> = HashMap::new();
        if let Some(phoenix) = config.phoenix {
            for (k, v) in phoenix {
                let port = &v.port.as_ref().unwrap().clone();
                let plugin = VePlugin::from(v);
                data.insert(port.to_string(), plugin.data.clone());
                plugins.push((k.to_string(), Box::new(plugin)));
            }
        }

        let port = config.port.unwrap_or_else(|| {
                warn!("port not set, falling back to 9104");
                9104
            });
        std::thread::spawn(move || {
            use std::net::{TcpListener, TcpStream};
            use std::io::{Write, Read, BufReader, BufRead};
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
            let data = Arc::new(Mutex::new(data));

            // accept connections and process them serially
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let data = data.clone();
                    std::thread::spawn(move || {
                        let stream: TcpStream = stream;
                        let mut reader = BufReader::new(stream);
                        //let mut line = String::new();
                        //reader.read_line(&mut line);
                        for line in reader.lines() {
                            let line = line.unwrap_or_else(|e| {
                                info!("line empty: {}", e);
                                String::new()
                            });
                            let line: Vec<&str> = line.split(':').collect();
                            if line.len() != 2 {
                                info!("line is not len 2");
                                continue;
                            }
                            let data = data.lock().map_err(|e| ()).and_then(|v| {
                                match v.get(line[0]) {
                                    Some(v) => Ok(v.clone()),
                                    None => Err(()),
                                }
                            });
                            info!("data: {:?}", data);
                        }
                    });
                }
                
            }
        });

        Ok(PluginRegistration::Multiple(plugins))
    }
}

collectd_plugin!(Manager);
