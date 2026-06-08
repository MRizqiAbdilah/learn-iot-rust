#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    prelude::*, rng::Rng
};


struct DataIklim {
    suhu: f32,
    kelembaban: f32,
}

fn baca_sensor(rng: &mut Rng) -> Result<DataIklim, &'static str> {
    let probabilitas_err = rng.random() % 10;

    if probabilitas_err < 2 {
        return Err("Kabel data DHT terputus...");
    }

    let suhu = 27.0 + ((rng.random() % 6) as f32);

    Ok(DataIklim {
        suhu: suhu,
        kelembaban: 70.0,
    })
}

enum StatusSuhu {
    Aman,
    Panas,
    ErrorSensor
}

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut rng = Rng::new(peripherals.RNG);


    let mut led_hijau = Output::new(io.pins.gpio4, Level::Low);
    let mut led_merah = Output::new(io.pins.gpio6, Level::Low);

    let mut status_sistem = StatusSuhu::Aman;

    esp_println::logger::init_logger_from_env();

    loop {
        match baca_sensor(&mut rng) {
            Ok(data) => {
                if data.suhu > 30.0 {
                    status_sistem = StatusSuhu::Panas;
                } else {
                    status_sistem = StatusSuhu::Aman
                }
            },
            Err(err) => {
                log::error!("Peringatan Sistem: {}\r", err);
                status_sistem = StatusSuhu::ErrorSensor;
            }
        }
        
        match status_sistem {
            // Aman: Nyalakan LED Hijau, matikan LED Merah. Tampilkan log peringatan yang sesuai.
            StatusSuhu::Aman => {
                led_hijau.set_high();
                led_merah.set_low();
                log::info!("Status: Aman, Pendingin Mati.\r");
            },
            // Panas: Matikan LED Hijau, nyalakan LED Merah (Anggap ini menyalakan kipas/pompa pendingin di greenhouse). Tampilkan log peringatan yang sesuai.
            StatusSuhu::Panas => {
                led_hijau.set_low();
                led_merah.set_high();
                log::info!("Status: Panas! Menyalakan Pendingin.\r");
            },
            StatusSuhu::ErrorSensor => {
                for _ in 0..5 {
                    delay.delay(500.millis());
                    led_hijau.set_high();
                    led_merah.set_high();
                    delay.delay(500.millis());
                    led_hijau.set_low();
                    led_merah.set_low();
                    log::warn!("Status: Error Sensor, Segera Perbaiki!\r")
                }
            }
        }
    
    delay.delay(2000.millis()); // Sensor iklim lambat, beri delay 2 detik
    }
}