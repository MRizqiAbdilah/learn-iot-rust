#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Io, Level, Output, Pull},
    prelude::*,
};

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led_merah = Output::new(io.pins.gpio6, Level::Low);

    let mut led_aktif: bool = false;

    let tombol = Input::new(io.pins.gpio2, Pull::Up);

    esp_println::logger::init_logger_from_env();

    loop {
        // membuat seolah" tombol itu menjadi toggle
        // tombol ditekan
        if tombol.is_low(){
            // Debouncing, jadi biar gak bisa spam
            delay.delay(100.millis());
            
            // Cek kembali, apakah tombol ditekan? 
            if tombol.is_low() {
                // ini akan membuat led_aktif menjadi true
                led_aktif = !led_aktif;

                // kalo led_aktif itu true
                if led_aktif {
                    // led_merah akan menyala dan akan menampilakn log Tombol: Aktif
                    led_merah.set_high();
                    log::info!("Tombol: Aktif\r")
                } else {
                    // led_merah akan mati dan akan menampilkan log Tombol: Tidak Aktif
                    led_merah.set_low();
                    log::info!("Tombol: Tidak Aktif\r")
                }

                // Jadi akan memberikan jeda, jika tombol dilepas
                // dilakukan agar tidak tidak bisa melakukan tekan-lepas tombol berkali"
                while tombol.is_low(){
                    delay.delay(50.millis());
                }
            }
        }
    }
}
