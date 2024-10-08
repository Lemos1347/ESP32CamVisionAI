use esp32cam_rs::{wifi, Camera, Flash, HttpClient, MultiPartForm};
use esp_idf_hal::{delay::FreeRtos, gpio::*, prelude::*};
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::prelude::Peripherals};
use log::{info, warn};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

// Pins used by camera
// static CAM_PIN_PWDN: i32 = 32;
static CAM_PIN_RESET: i32 = -1;
// static CAM_PIN_XCLK: i32 = 0;
// static CAM_PIN_SIOD: i32 = 26;
// static CAM_PIN_SIOC: i32 = 27;
// static CAM_PIN_D7: i32 = 35;
// static CAM_PIN_D6: i32 = 34;
// static CAM_PIN_D5: i32 = 39;
// static CAM_PIN_D4: i32 = 36;
// static CAM_PIN_D3: i32 = 21;
// static CAM_PIN_D2: i32 = 19;
// static CAM_PIN_D1: i32 = 18;
// static CAM_PIN_D0: i32 = 5;
// static CAM_PIN_VSYNC: i32 = 25;
// static CAM_PIN_HREF: i32 = 23;
// static CAM_PIN_PCLK: i32 = 22;
//
#[toml_cfg::toml_config]
pub struct Config {
    #[default("WifiName")]
    wifi_ssid: &'static str,
    #[default("WifiPassword")]
    wifi_psk: &'static str,
    #[default("")]
    server_url: &'static str,
    #[default(false)]
    use_flash: bool,
    #[default(32)]
    flash_brightness: u8,
}

fn send_images(buffer: &Arc<(Mutex<VecDeque<Arc<Box<[u8]>>>>, Condvar)>) {
    info!("Setting up http_client!");
    let mut http_client = HttpClient::new().expect("Unable to start HttpClient!");
    info!("Http_client ok!");

    let uri = format!("{}/post", CONFIG.server_url);

    let mut form_data = MultiPartForm::new();
    let headers = [("Content-type", form_data.content_type)];

    let (lock, cvar) = &**buffer;
    loop {
        let mut buffer_guard = lock.lock().unwrap();

        // Wait for images in buffer
        while buffer_guard.is_empty() {
            info!("Waiting for images");
            buffer_guard = cvar.wait(buffer_guard).unwrap();
        }

        // Take a buffer from queue
        if let Some(image_data) = buffer_guard.pop_front() {
            drop(buffer_guard); // Free buffer before sending HTTP request

            form_data.add_file("file", &*image_data);

            match http_client.post(&uri, &headers, &form_data.wrap_up()) {
                Ok(_) => info!("Image sent successfully!"),
                Err(err) => {
                    warn!("Unable to reach server! Err: {}", err.to_string());
                    FreeRtos::delay_ms(5000);
                    info!("Trying to reach again!");
                }
            };
        }
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let app_config = CONFIG;

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    info!("Initialyzing esp32 cam");
    let cam = Camera::new(
        peripherals.pins.gpio32,
        unsafe { AnyIOPin::new(CAM_PIN_RESET) },
        peripherals.pins.gpio0,
        peripherals.pins.gpio26,
        peripherals.pins.gpio27,
        peripherals.pins.gpio5,
        peripherals.pins.gpio18,
        peripherals.pins.gpio19,
        peripherals.pins.gpio21,
        peripherals.pins.gpio36,
        peripherals.pins.gpio39,
        peripherals.pins.gpio34,
        peripherals.pins.gpio35,
        peripherals.pins.gpio25,
        peripherals.pins.gpio23,
        peripherals.pins.gpio22,
    )
    .expect("Unable to initialyze camera");

    let sensor = cam.sensor();

    sensor.set_gain_ctrl(true)?;
    sensor.set_exposure_ctrl(true)?;
    sensor.set_awb_gain(true)?;
    sensor.set_brightness(0)?;

    info!("Camera initialyzed!");

    info!("Initialyzing esp32 flash");
    let mut flash = Flash::new(
        peripherals.ledc.channel0,
        peripherals.ledc.timer1,
        peripherals.pins.gpio4,
        5.kHz(),
    )?;
    info!("Flash initialyzed!");

    if app_config.use_flash {
        flash.activate(Some(app_config.flash_brightness))?;
    }

    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    let buffer = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));

    let buffer_cloned = Arc::clone(&buffer);
    let _send_image_thread = thread::spawn(move || {
        send_images(&buffer_cloned);
    });

    let (lock, cvar) = &*buffer;
    loop {
        let mut buffer_guard = lock.lock().unwrap();
        if buffer_guard.len() >= 5 {
            warn!("Buffer is full, waiting for space...");
            cvar.notify_one(); // Notify another thread to send the photo
            continue;
        }
        match cam.get_framebuffer() {
            Some(frame) => {
                buffer_guard.push_back(Arc::new(Box::from(frame.data()))); // Add frame to buffer
                info!("Image added to buffer. Buffer size: {}", buffer_guard.len());
                cvar.notify_one();
            }
            None => warn!("Unable to get frame!"),
        };
    }

    // _send_image_thread.join().unwrap();
}
