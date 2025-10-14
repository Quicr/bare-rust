// Hactar command definitions
// These will be implemented in Milestone 2

use std::collections::HashMap;

// Command map - maps command strings to their byte representations
pub fn get_command_map() -> HashMap<&'static str, Vec<u8>> {
    let mut map = HashMap::new();

    // Basic commands (Type + 4-byte length = 5 bytes total)
    map.insert("version", vec![0, 0, 0, 0, 0]);
    map.insert("who are you", vec![1, 0, 0, 0, 0]);
    map.insert("hard reset", vec![2, 0, 0, 0, 0]);
    map.insert("reset", vec![3, 0, 0, 0, 0]);
    map.insert("reset ui", vec![4, 0, 0, 0, 0]);
    map.insert("reset net", vec![5, 0, 0, 0, 0]);
    map.insert("flash ui", vec![6, 0, 0, 0, 0]);
    map.insert("flash net", vec![7, 0, 0, 0, 0]);
    map.insert("enable logs", vec![8, 0, 0, 0, 0]);
    map.insert("enable ui logs", vec![9, 0, 0, 0, 0]);
    map.insert("enable net logs", vec![10, 0, 0, 0, 0]);
    map.insert("disable logs", vec![11, 0, 0, 0, 0]);
    map.insert("disable ui logs", vec![12, 0, 0, 0, 0]);
    map.insert("disable net logs", vec![13, 0, 0, 0, 0]);
    map.insert("default logging", vec![14, 0, 0, 0, 0]);

    map
}

#[derive(Debug, Clone, Copy)]
pub enum BypassTarget {
    Ui = 15,
    Net = 16,
    Loopback = 17,
}

impl std::str::FromStr for BypassTarget {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ui" => Ok(Self::Ui),
            "net" => Ok(Self::Net),
            "loopback" => Ok(Self::Loopback),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChipCommand {
    pub id: u8,
    pub num_params: usize,
}

pub fn get_ui_command_map() -> HashMap<&'static str, ChipCommand> {
    let mut map = HashMap::new();
    map.insert("version", ChipCommand { id: 0, num_params: 0 });
    map.insert("clear_config", ChipCommand { id: 1, num_params: 0 });
    map.insert("set_sframe", ChipCommand { id: 2, num_params: 1 });
    map.insert("get_sframe", ChipCommand { id: 3, num_params: 0 });
    map
}

pub fn get_net_command_map() -> HashMap<&'static str, ChipCommand> {
    let mut map = HashMap::new();
    map.insert("version", ChipCommand { id: 0, num_params: 0 });
    map.insert("clear_storage", ChipCommand { id: 1, num_params: 0 });
    map.insert("set_ssid", ChipCommand { id: 2, num_params: 2 });
    map.insert("get_ssid_names", ChipCommand { id: 3, num_params: 0 });
    map.insert("get_ssid_passwords", ChipCommand { id: 4, num_params: 0 });
    map.insert("clear_ssids", ChipCommand { id: 5, num_params: 0 });
    map.insert("set_moq_url", ChipCommand { id: 6, num_params: 1 });
    map.insert("get_moq_url", ChipCommand { id: 7, num_params: 0 });
    map
}
