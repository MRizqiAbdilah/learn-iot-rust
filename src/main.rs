#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Io, Level, Output, Pull},
    prelude::*,
};

enum StatusAir {
    Normal,
    Rendah,
    Kritis
}

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // merah untuk indikator alarm
    let mut led_merah = Output::new(io.pins.gpio6, Level::Low);

    // hijau untuk indikator normal
    let mut led_hijau = Output::new(io.pins.gpio5, Level::Low);

    let tombol = Input::new(io.pins.gpio2, Pull::Up);

    // Set status_sistem awal yaitu Normal
    let mut status_sistem = StatusAir::Normal;

    // Ini merupakan awal, sebelum memasuki loop
    // indikator normal dinyalakan
    log::info!("Status: Normal. Air cukup.\r");
    led_hijau.set_high();

    esp_println::logger::init_logger_from_env();
    
    loop {
        // membuat seolah" tombol itu menjadi toggle
        // tombol ditekan
        if tombol.is_low(){
            // Debouncing, jadi biar gak bisa spam
            delay.delay(50.millis());
            
            // Cek kembali, apakah tombol ditekan? 
            if tombol.is_low() {
                // ini akan membuat status_sitem berubah
                // normal ke rendah, rendah ke kritis, kritis ke normal, dst
                status_sistem = match status_sistem {
                    StatusAir::Normal => StatusAir::Rendah,
                    StatusAir::Rendah => StatusAir::Kritis,
                    StatusAir::Kritis => StatusAir::Normal
                };

                // Jadi akan memberikan jeda, jika tombol dilepas
                // dilakukan agar tidak tidak bisa melakukan tekan-lepas tombol berkali"
                while tombol.is_low(){
                    delay.delay(10.millis());
                }
            }

            // tergantung pada status_sistem state yang dilakukan akan berbeda"
            match status_sistem {
                StatusAir::Normal => {
                    // indikator normal akan dinyalakan
                    log::info!("Status: Normal. Air cukup.\r");
                    led_hijau.set_high();
                    led_merah.set_low();
                },
                StatusAir::Rendah => {
                    // indikator normal dan alarm akan dimatikan, akan menyalakan pompa
                    log::info!("Status: Rendah. Sedang menyalakan pompa.\r");
                    led_hijau.set_low();
                },
                StatusAir::Kritis => {
                    // indikator alarm dinyalakan, pompa dimatikan
                    log::info!("Status: Kritis. Pompa dimatikan, Alarm dinyalakan.\r");
                    led_merah.set_high();
                }
            }
        }
    }
}
