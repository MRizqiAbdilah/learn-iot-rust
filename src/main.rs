#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    prelude::*, rng::Rng
};

// 1. Deklarasi Struct Data
struct DataIklim {
    suhu: f32,
    kelembaban: f32,
}

// 2. Fungsi Mocking untuk menstimulasi pembacaan sensor
// Nanti ini akan kita ganti dengan driver DHT sungguhan
fn baca_sensor() -> DataIklim {
    // Mengembalikan instance dari struct
    DataIklim {
        suhu: 26.0,       // Coba ubah angka ini nanti saat latihan
        kelembaban: 70.0,
    }
}

// 3. Modifikasi FSM Enum untuk Suhu
enum StatusSuhu {
    Aman,
    Panas,
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
        // A. Proses Input: Membaca struct dari sensor
        let mut data_saat_ini = baca_sensor();
        let angka_acak = rng.random();

        let fluktuasi = (angka_acak % 6) as f32;

        log::info!("Membaca DHT... Suhu: {} C, Kelembaban: {} %\r", data_saat_ini.suhu, data_saat_ini.kelembaban);

        data_saat_ini.suhu = data_saat_ini.suhu + fluktuasi;
        log::info!("Suhu setelah fluktuasi: {} C \r", data_saat_ini.suhu);


        // B. Proses Transisi State (LATIHAN ANDA DI SINI)
        // ...
        if data_saat_ini.suhu > 30.0 {
            status_sistem = StatusSuhu::Panas;
        } else {
            status_sistem = StatusSuhu::Aman
        }

        // C. Proses Output (LATIHAN ANDA DI SINI)
        // ...
        match status_sistem {
            // Aman: Nyalakan LED Hijau, matikan LED Merah. Tampilkan log peringatan yang sesuai.
            StatusSuhu::Aman => {
                led_hijau.set_high();
                led_merah.set_low();
                log::info!("Status: Aman, Matikan pendingin...\r");
            },
            // Panas: Matikan LED Hijau, nyalakan LED Merah (Anggap ini menyalakan kipas/pompa pendingin di greenhouse). Tampilkan log peringatan yang sesuai.
            StatusSuhu::Panas => {
                led_hijau.set_low();
                led_merah.set_high();
                log::info!("Status: Panas, nyalakan pendingin...\r");
            }
        }

        delay.delay(2000.millis()); // Sensor iklim lambat, beri delay 2 detik
    }
}