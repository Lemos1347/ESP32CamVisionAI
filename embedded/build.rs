#[toml_cfg::toml_config]
pub struct Config {
    #[default("WifiName")]
    wifi_ssid: &'static str,
    #[default("WifiPassword")]
    wifi_psk: &'static str,
    #[default("")]
    server_url: &'static str,
    #[default(false)]
    use_flash: bool,
    #[default(32)]
    flash_brightness: u8,
}

fn main() {
    if !std::path::Path::new("cfg.toml").exists() {
        panic!("You need to create a `cfg.toml` file with your Wi-Fi credentials! Use `cfg.toml.example` as a template.");
    }

    let app_config = CONFIG;
    if app_config.wifi_ssid == "WifiName" || app_config.wifi_psk == "WifiPassword" {
        panic!("You need to set the Wi-Fi credentials in `cfg.toml`!");
    }

    embuild::espidf::sysenv::output();
}
