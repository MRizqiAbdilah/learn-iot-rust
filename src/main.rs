#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, gpio::{Io, Level, Output}, prelude::*};

fn nyalakan_alarm(delay: &Delay, pin_alarm: &mut Output<'_>) {
    for _ in 0..5 {
        pin_alarm.set_high();
        delay.delay(200.millis());
        pin_alarm.set_low();
        delay.delay(200.millis());
    }
}

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // led_biru = 5, led_merah = 6

    // indikator pompa
    let mut led_biru = Output::new(io.pins.gpio5, Level::Low);
    // indikator alarm
    let mut led_merah = Output::new(io.pins.gpio6, Level::Low);

    esp_println::logger::init_logger_from_env();

    loop {
        log::info!("Fase Pengairan: Pompa menyala selama 4 detik...\r");
        led_biru.set_high();
        led_merah.set_low();
        delay.delay(4000.millis());

        log::info!("Fase Peringatan Level Air: Alarm nyala sebanyak 5 kali...\r");
        led_biru.set_low();
        nyalakan_alarm(&delay, &mut led_merah);

        log::info!("Fase Jeda: semua indikator mati dan akan kembali ke fase Pengairan dalam 2 detik... \r");
        led_merah.set_low();
        delay.delay(2000.millis());
    }
}
