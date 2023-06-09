use anyhow::Context;
use embedded_svc::wifi::{
    AccessPointConfiguration, AuthMethod, ClientConfiguration, Configuration,
};
use esp_idf_hal::modem::WifiModemPeripheral;
use esp_idf_svc::{eventloop::EspEventLoop, wifi::EspWifi};

pub fn connect<'d>(
    modem: &'d mut impl WifiModemPeripheral,
    wifi_ssid: &str,
    wifi_pass: &str,
) -> anyhow::Result<EspWifi<'d>> {
    let sysloop = EspEventLoop::take()?;
    let mut wifi = EspWifi::new(modem, sysloop, None)?;

    println!("Wifi created, scanning available networks...");

    let available_networks = wifi.scan()?;
    let target_network = available_networks
        .iter()
        .find(|network| network.ssid == wifi_ssid)
        .with_context(|| format!("Failed to detect the target network ({wifi_ssid})"))?;

    println!("Scan successfull, found '{wifi_ssid}', with config: {target_network:#?}");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: wifi_ssid.into(),
        password: wifi_pass.into(),
        auth_method: target_network.auth_method,
        bssid: Some(target_network.bssid),
        channel: Some(target_network.channel),
    }))?;

    wifi.start()?;
    wifi.connect()?;

    Ok(wifi)
}

pub fn start_access_point<'d>(
    modem: &'d mut impl WifiModemPeripheral,
    wifi_ssid: &str,
    wifi_pass: &str,
) -> anyhow::Result<EspWifi<'d>> {
    let sysloop = EspEventLoop::take()?;
    let mut wifi = EspWifi::new(modem, sysloop, None)?;

    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: wifi_ssid.into(),
        password: wifi_pass.into(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    wifi.start()?;

    Ok(wifi)
}
