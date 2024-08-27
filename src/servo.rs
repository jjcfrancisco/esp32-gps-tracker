use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution};
use std::sync::{Arc, Mutex};
use esp_idf_hal::prelude::Hertz;
use esp_idf_hal::ledc::{TIMER0, CHANNEL3};
use esp_idf_hal::gpio;
use esp_idf_hal::delay::FreeRtos;

pub fn driver(timer: TIMER0, channel3: CHANNEL3, gpio13: gpio::Gpio13) -> Arc<Mutex<LedcDriver<'static>>> {
    // Servo
    let timer_driver = LedcTimerDriver::new(
        timer,
        &TimerConfig::default()
            .frequency(Hertz(50))
            .resolution(Resolution::Bits14),
    )
    .unwrap();

    let servo_driver = Arc::new(Mutex::new(
        LedcDriver::new(
            channel3,
            timer_driver,
            gpio13
        )
        .unwrap(),
    ));

    // Give servo some time to update
    FreeRtos::delay_ms(500);

    servo_driver

}

pub fn calculate_ranges(driver: &LedcDriver) -> (u32, u32) {
    // Get Max Duty and Calculate Upper and Lower Limits for Servo
    let max_duty = driver.get_max_duty();
    let min_limit = max_duty * 25 / 1000;
    let max_limit = max_duty * 125 / 1000;
    (min_limit, max_limit)
}

// Function that maps one range to another
pub fn map(x: u32, in_min: u32, in_max: u32, out_min: u32, out_max: u32) -> u32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}
