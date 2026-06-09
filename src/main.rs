#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Io, Level, Output, Pull},
    prelude::*, timer::systimer::SystemTimer,
};

// Enum FSM yang merepresentasikan status pompa dengan Histeresis
#[derive(PartialEq, Debug)] // Ditambahkan agar enum bisa dibandingkan nilainya
enum StatusPompa {
    Menunggu, // Pompa mati, menunggu air surut
    Mengisi,  // Pompa hidup, sedang mengisi bak
}


// (Salin FUNGSI hitung_jarak_cm dari Hari ke-9 di sini)
// (Salin FUNGSI baca_ultrasonik dari Hari ke-9 di sini)
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
    let mut delay = Delay::new();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut pin_trig = Output::new(io.pins.gpio3, Level::Low);
    let pin_echo = Input::new(io.pins.gpio1, Pull::Down);
    
    // Inisialisasi pin untuk mengontrol Relay
    let mut relay_pompa = Output::new(io.pins.gpio7, Level::Low);

    // Status awal pompa
    let mut status_pompa = StatusPompa::Menunggu;

    esp_println::logger::init_logger_from_env();
    log::info!("Sistem Kontrol Pompa Air Aktif...\r");

    loop {
        match baca_ultrasonik(&mut pin_trig, &pin_echo, &mut delay) {
            Ok(jarak) => {
                log::info!("Jarak Permukaan: {:.2} cm\r", jarak);

                // (TANTANGAN ANDA: Lakukan logika transisi state dengan Histeresis di sini)
                /* Contoh alur pemikiran Histeresis:
                   Jika status saat ini Menunggu DAN jarak >= 20.0 -> ubah ke Mengisi
                   Jika status saat ini Mengisi DAN jarak <= 10.0 -> ubah ke Menunggu
                */
                if status_pompa == StatusPompa::Menunggu && jarak >= 20.0 {
                    status_pompa = StatusPompa::Mengisi;
                } else if status_pompa == StatusPompa::Mengisi && jarak <= 10.0 {
                    status_pompa = StatusPompa::Menunggu;
                }
            }
            Err(err) => {
                log::error!("Error Sensor: {}\r", err);
                // Matikan pompa demi keamanan jika sensor rusak
                status_pompa = StatusPompa::Menunggu;
            }
        }

        // (TANTANGAN ANDA: Eksekusi state pada relay)
        match status_pompa {
            StatusPompa::Menunggu => {
                // Matikan relay, tulis log
                log::info!("Status: Menunggu, Mematikan Pompa Air\r");
                relay_pompa.set_low();
            }
            StatusPompa::Mengisi => {
                // Nyalakan relay, tulis log
                log::info!("Status: Mengisi, Mengaktifkan Pompa Air\r");
                relay_pompa.set_high();
            }
        }

        delay.delay(1000.millis());
    }
}