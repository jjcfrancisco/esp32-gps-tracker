use crate::Result;

use esp_idf_hal::peripherals::Peripherals;
use futures::executor::block_on;
use std::{thread::sleep, time::Duration};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::server::{Configuration, EspHttpServer},
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
};

use embedded_svc::{http::Method, io::Write};

use esp_idf_hal::delay::FreeRtos;

use crate::servo;
use crate::wifi;

pub fn run() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();

    // Initialise servo
    let servo_driver = servo::driver(
        peripherals.ledc.timer0,
        peripherals.ledc.channel3,
        peripherals.pins.gpio13,
    );

    // Initialise Wi-Fi
    let mut wifi = wifi::get(peripherals.modem, sys_loop, Some(nvs), timer_service)?;

    // Connect to Wi-Fi
    block_on(wifi::connect(&mut wifi))?;
    log::info!("Connected to Wi-Fi");

    // Ping Wi-Fi
    wifi::ping(&wifi)?;

    // API
    let mut server = EspHttpServer::new(&Configuration::default())?;
    server.fn_handler("/", Method::Get, |request| {
        let html = "I hear you!";
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    server.fn_handler("/move", Method::Post, move |request| {
        let html = "Moving...";
        let params = request.uri().split('?').collect::<Vec<&str>>();
        let degrees = params[1].split('=').collect::<Vec<&str>>()[1]
            .parse::<u32>()
            .unwrap();
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        let (min_limit, max_limit) = servo::calculate_ranges(&servo_driver.lock().unwrap());
        servo_driver
            .lock()
            .unwrap()
            .set_duty(servo::map(degrees, 0, 360, min_limit, max_limit))
            .unwrap();
        // Give servo some time to update
        FreeRtos::delay_ms(12);
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    loop {
        sleep(Duration::from_secs(1));
    }
}
