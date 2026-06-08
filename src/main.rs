#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Io, Level, Output, OutputOpenDrain, Pull},
    prelude::*,
};

// Import crate driver DHT22
use dht_sensor::{dht22};

enum StatusSuhu {
    Aman,
    Panas,
    ErrorSensor
}

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    // Kita buat delay mutable (bisa diubah) karena dht_sensor membutuhkannya untuk menghitung timing
    let mut delay = Delay::new();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Konfigurasi pin DHT22 (GPIO 8) sebagai Open-Drain dengan Internal Pull-Up
    let mut dht_pin = OutputOpenDrain::new(io.pins.gpio8, Level::High, Pull::Up);

    // merah = 6, hijau = 4
    let mut led_merah = Output::new(io.pins.gpio6, Level::Low);
    let mut led_hijau = Output::new(io.pins.gpio4, Level::Low);

    let mut status_sistem = StatusSuhu::Aman;
    
    esp_println::logger::init_logger_from_env();
    log::info!("Sistem Hidroponik Menyala. Memulai kalibrasi DHT22...\r");

     // Sensor DHT butuh waktu 1-2 detik setelah power-on sebelum bisa dibaca
    delay.delay(2000.millis());
    loop {
        // Crate dht_sensor sudah otomatis mengembalikan enum Result!
        // Fungsinya akan membaca pin secara langsung.
        match dht22::blocking::read(&mut delay, &mut dht_pin) {
            Ok(data) => {
                // data.temperature bertipe f32
                // data.relative_humidity bertipe f32
                log::info!("Sensor Fisik -> Suhu: {} C, Kelembaban: {} %\r", data.temperature, data.relative_humidity);

                // (TANTANGAN ANDA: Lakukan logika transisi State di sini)
                if data.temperature > 30.0 {
                    status_sistem = StatusSuhu::Panas;
                } else {
                    status_sistem = StatusSuhu::Aman;
                }
            }
            Err(_error) => {
                // Crate ini menghasilkan error tipe enum khusus, kita konversi saja ke log manual
                log::error!("Peringatan Sistem: Gagal membaca sensor DHT fisik!\r");
                
                // (TANTANGAN ANDA: Lakukan transisi state ErrorSensor di sini)
                status_sistem = StatusSuhu::ErrorSensor;
            }
        }

         // (TANTANGAN ANDA: Lakukan eksekusi output LED berdasarkan State di sini)
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
        // Delay 2 detik (DHT22 tidak boleh dibaca lebih cepat dari 2 detik sekali)
        delay.delay(2000.millis());
    }
}
