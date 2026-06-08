#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Io, Level, Output, OutputOpenDrain, Pull},
    prelude::*, timer::systimer::SystemTimer,
};

// Import crate driver DHT22
use dht_sensor::{dht22};

enum StatusSuhu {
    Aman,
    Panas,
    ErrorSensor
}

enum StatusAir {
    Cukup,
    Surut,
    Habis
}

fn hitung_jarak_cm(durasi_us: u64) -> f32 {
    let jarak: f32 = (durasi_us as f32 * 0.0343) / 2.0;
    return jarak
}

fn baca_ultrasonik(trig: &mut Output<'_>, echo: &Input<'_>, delay: &mut Delay ) -> Result<f32, &'static str> {
    // 1. Tembak sinyal trigger 10 mikrodetik
    trig.set_low();
    delay.delay(2.micros());
    trig.set_high();
    delay.delay(10.micros());
    trig.set_low();

    // 2. Tunggu pin echo naik menjadi high (Mulai mencatat waktu)
    let mut timeout = 100_000;
    while echo.is_low() {
        timeout -= 1;
        if timeout == 0 {
            return Err("Timeout! Sensor tidak merespons (Echo tidak High)");
        }
    }

    let waktu_mulai = SystemTimer::now();

    // 3. Tunggu pin Echo turun jadi Low (Selesai mencatat waktu pantulan)
    timeout = 100_000;
    while echo.is_high() {
        timeout -= 1;
        if timeout == 0 { return Err("Timeout! Objek terlalu jauh (Echo tidak Low)"); }
    }
    let waktu_selesai = SystemTimer::now();

    // 4. Konversi Ticks prosesor ke Mikrodetik
    let durasi_ticks = waktu_selesai - waktu_mulai;
    let durasi_us = durasi_ticks / 16;

    let jarak_cm = hitung_jarak_cm(durasi_us);
    Ok(jarak_cm)
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

    // Inisialisasi HC-SR04
    let mut pin_trig = Output::new(io.pins.gpio3, Level::Low);

    // Pin echo sebagai input dengan internal Pull-Down untuk mencegah floating
    let pin_echo = Input::new(io.pins.gpio1, Pull::Down);

    // merah = 6, hijau = 4
    let mut led_merah = Output::new(io.pins.gpio6, Level::Low);
    let mut led_hijau = Output::new(io.pins.gpio4, Level::Low);

    let mut status_suhu = StatusSuhu::Aman;
    let mut status_air = StatusAir::Cukup;
    
    esp_println::logger::init_logger_from_env();
    log::info!("Sistem Hidroponik Menyala. Memulai kalibrasi DHT22...\r");
    
    // Sensor DHT butuh waktu 1-2 detik setelah power-on sebelum bisa dibaca
    delay.delay(2000.millis());
    
    loop {
        match baca_ultrasonik(&mut pin_trig, &pin_echo, &mut delay) {
            Ok(jarak) => {
                log::info!("Jarak Permukaan Air: {:.2} cm\r", jarak);

                if jarak < 10.0 {
                    status_air = StatusAir::Cukup;
                } else if jarak >= 10.0 && jarak < 20.0 {
                    status_air = StatusAir::Surut;
                } else {
                    status_air = StatusAir::Habis;
                }
            }
            Err(err) => {
                log::error!("Pesan sistem: {}\r", err);
            }
        }

        match status_air {
            StatusAir::Cukup => {
                led_hijau.set_high();
                led_merah.set_low();
                log::info!("Aman: Tangki Air Penuh.\r");
            },
            StatusAir::Surut => {
                led_hijau.set_low();
                led_merah.set_low();
                log::info!("Peringatan: Air mulai surut.\r");
            },
            StatusAir::Habis => {
                led_hijau.set_low();
                led_merah.set_high();
                log::info!("KRITIS: Air habis! Isi ulang sekarang.\r");
            }
        }

        delay.delay(1000.millis());

        // Crate dht_sensor sudah otomatis mengembalikan enum Result!
        // Fungsinya akan membaca pin secara langsung.
        match dht22::blocking::read(&mut delay, &mut dht_pin) {
            Ok(data) => {
                // data.temperature bertipe f32
                // data.relative_humidity bertipe f32
                log::info!("Sensor Fisik -> Suhu: {} C, Kelembaban: {} %\r", data.temperature, data.relative_humidity);
            
                // (TANTANGAN ANDA: Lakukan logika transisi State di sini)
                if data.temperature > 30.0 {
                    status_suhu = StatusSuhu::Panas;
                } else {
                    status_suhu = StatusSuhu::Aman;
                }
                
            }
            Err(_error) => {
                // Crate ini menghasilkan error tipe enum khusus, kita konversi saja ke log manual
                log::error!("Peringatan Sistem: Gagal membaca sensor DHT fisik!\r");
                
                // (TANTANGAN ANDA: Lakukan transisi state ErrorSensor di sini)
                status_suhu = StatusSuhu::ErrorSensor;
            }
        }
        

        // (TANTANGAN ANDA: Lakukan eksekusi output LED berdasarkan State di sini)
        match status_suhu {
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
            StatusSuhu::ErrorSensor  => {
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