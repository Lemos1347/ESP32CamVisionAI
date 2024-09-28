use esp_idf_hal::{
    delay::FreeRtos,
    gpio::OutputPin,
    ledc::{config, LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver},
    peripheral::Peripheral,
    prelude::*,
};

pub struct Flash<'d> {
    led_control: LedcDriver<'d>,
}

impl<'d> Flash<'d> {
    pub fn new<T, C, P>(
        channel: impl Peripheral<P = C> + 'd,
        timer: impl Peripheral<P = T> + 'd,
        pin: impl Peripheral<P = P> + 'd,
        frequency: KiloHertz,
    ) -> anyhow::Result<Self>
    where
        T: LedcTimer + 'd,
        C: LedcChannel<SpeedMode = <T as LedcTimer>::SpeedMode>,
        P: OutputPin,
    {
        let driver = LedcDriver::new(
            channel,
            LedcTimerDriver::new(
                timer,
                &config::TimerConfig::new().frequency(frequency.into()),
            )?,
            pin,
        )?;
        Ok(Flash {
            led_control: driver,
        })
    }

    pub fn activate(&mut self, brightness: Option<u8>) -> anyhow::Result<()> {
        let led_brightness = match brightness {
            Some(v) => v,
            None => 255,
        };
        self.led_control.set_duty(led_brightness.into())?;
        Ok(())
    }

    pub fn deactivate(&mut self) -> anyhow::Result<()> {
        self.led_control.set_duty(0)?;
        Ok(())
    }

    pub fn blink(&mut self, times: u8, brightness: Option<u8>) -> anyhow::Result<()> {
        for _ in 0..times {
            self.activate(brightness)?;
            FreeRtos::delay_ms(750);
            self.deactivate()?;
            FreeRtos::delay_ms(500);
        }

        Ok(())
    }
}
