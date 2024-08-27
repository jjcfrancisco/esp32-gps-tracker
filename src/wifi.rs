use crate::Result;
use embedded_svc::wifi::{ClientConfiguration, Configuration as wifiConfiguration};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, timer::EspTaskTimerService,
};
use heapless;
use std::str::FromStr;
use esp_idf_svc::{
    ping::Configuration as pingConfiguration,
    ping::EspPing,
};

pub async fn connect(wifi: &mut AsyncWifi<EspWifi<'static>>) -> Result<()> {
    wifi.start().await?;
    log::info!("Wifi started");

    wifi.connect().await?;
    log::info!("Wifi connected");

    wifi.wait_netif_up().await?;
    log::info!("Wifi netif up");
    Ok(())
}

pub fn get(
    modem: Modem,
    sys_loop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
    timer_service: EspTaskTimerService,
) -> Result<AsyncWifi<EspWifi<'static>>> {
    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), nvs)?,
        sys_loop.clone(),
        timer_service,
    )?;

    const SSID: &str = env!("SSID");
    const PASSWORD: &str = env!("PASSWORD");

    wifi.set_configuration(&wifiConfiguration::Client(ClientConfiguration {
        ssid: heapless::String::<32>::from_str(SSID).unwrap(),
        password: heapless::String::<64>::from_str(PASSWORD).unwrap(),
        ..Default::default()
    }))
    .unwrap();

    Ok(wifi)
}

pub fn ping(wifi: &AsyncWifi<EspWifi<'static>>) -> Result<()> {
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    println!("Wifi DHCP info: {:?}", ip_info);

    // Ping
    EspPing::default().ping(ip_info.subnet.gateway, &pingConfiguration::default())?;

    Ok(())
}
