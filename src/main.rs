mod config;

fn main() {
    let mut cfg = config::Config::new("./config/settings.json", "./config/channels.json");
    cfg.add_channel("testname".to_string(), "testchannel".to_string());
    cfg.add_channel("testname2".to_string(), "testchannel".to_string());
}
