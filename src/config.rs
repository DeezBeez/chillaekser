use std::fs::{self, File, OpenOptions};
use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::Path;

pub struct Config {
    pub settings: HashMap<String, String>,
    pub channel_names: HashMap<String, String>,
    settings_file: File,
    channel_file: File
}

impl Config {
    pub fn new(settings_path: &str, channel_path: &str) -> Self {
        let mut set: HashMap<String, String>  = HashMap::new();
        let mut can: HashMap<String, String> = HashMap::new();

        let s_p = Path::new(settings_path);
        fs::create_dir(s_p.parent().unwrap()).unwrap();

        let c_p = Path::new(channel_path);
        fs::create_dir(c_p.parent().unwrap()).unwrap();

        let mut open_options = OpenOptions::new();
        open_options.append(false).read(true).write(true).create(true);
        let mut settings_file: File = open_options.open(settings_path).expect("Failed to open settings file!");

        let mut buffer = String::new();
        settings_file.read_to_string(&mut buffer).unwrap();
        if buffer.is_empty() {
            buffer = serde_json::ser::to_string(&set).unwrap();
        }
        set = serde_json::from_str(&buffer).unwrap();

        let mut channel_file: File = open_options.open(channel_path).expect("Failed to open channels file!");
        buffer = String::new();
        channel_file.read_to_string(&mut buffer).unwrap();
        if buffer.is_empty() {
            buffer = serde_json::ser::to_string(&can).unwrap();
        }        
        can = serde_json::from_str(&buffer).unwrap();
        
        Config { 
            settings: set, 
            channel_names: can,
            settings_file: settings_file,
            channel_file: channel_file
        }
    }

    pub fn add_channel(&mut self, username: String, channel_name: String) {
        self.channel_names.insert(username, channel_name);
        let v = serde_json::ser::to_string_pretty(&self.channel_names).expect("Error serializing channel names").trim().to_string();
        println!("{v}");
        self.channel_file.set_len(0).unwrap();
        self.channel_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.channel_file.write_all(v.as_bytes()).expect("Failed to write to channel file!");
    }
}