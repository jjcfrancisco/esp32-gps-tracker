use crate::Result;

use esp_idf_hal::peripherals::Peripherals;
use futures::executor::block_on;
use std::{str::FromStr, thread::sleep, time::Duration};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::server::{Configuration, EspHttpServer},
    nvs::EspDefaultNvsPartition,
    ping::Configuration as pingConfiguration,
    ping::EspPing,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi},
};

use embedded_svc::wifi::{ClientConfiguration, Configuration as wifiConfiguration};
use embedded_svc::{http::Method, io::Write};
use heapless;

pub fn test() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop.clone(),
        timer_service.clone(),
    )?;

    const SSID: &str = env!("SSID");
    const PASSWORD: &str = env!("PASSWORD");

    wifi.set_configuration(&wifiConfiguration::Client(ClientConfiguration {
        ssid: heapless::String::<32>::from_str(SSID).unwrap(),
        password: heapless::String::<64>::from_str(PASSWORD).unwrap(),
        ..Default::default()
    }))
    .unwrap();

    block_on(connect_wifi(&mut wifi))?;
    log::info!("Connected to Wi-Fi");

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    println!("Wifi DHCP info: {:?}", ip_info);

    // Ping
    EspPing::default().ping(ip_info.subnet.gateway, &pingConfiguration::default())?;

    // Server
    let mut server = EspHttpServer::new(&Configuration::default())?;
    server.fn_handler("/", Method::Get, |request| {
        let html = "I hear you!";
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    loop {
        sleep(Duration::from_secs(1));
    }
}

async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>) -> Result<()> {
    wifi.start().await?;
    log::info!("Wifi started");

    wifi.connect().await?;
    log::info!("Wifi connected");

    wifi.wait_netif_up().await?;
    log::info!("Wifi netif up");
    Ok(())
}
