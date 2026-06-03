#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, gpio::{Io, Level, Output}, prelude::*};

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // led_hijau = 4, led_kuning = 5, led_merah = 6
    let mut led_hijau = Output::new(io.pins.gpio4, Level::High);
    let mut led_kuning = Output::new(io.pins.gpio5, Level::Low);
    let mut led_merah = Output::new(io.pins.gpio6, Level::Low);

    esp_println::logger::init_logger_from_env();

    loop {
        log::info!("Status: Lampu Hijau Menyala!\r");
        log::info!("Waktu: 60 detik");
        led_hijau.set_high();
        led_kuning.set_low();
        led_merah.set_low();

        delay.delay(60000.millis());

        log::info!("Status: Lampu Kuning Menyala!\r");
        log::info!("Waktu: 5 detik");
        led_hijau.set_low();
        led_kuning.set_high();
        led_merah.set_low();

        delay.delay(5000.millis());

        log::info!("Status: Lampu Merah Menyala!\r");
        log::info!("Waktu: 60 detik");
        led_hijau.set_low();
        led_kuning.set_low();
        led_merah.set_high();

        delay.delay(6000.millis());

    }
}
