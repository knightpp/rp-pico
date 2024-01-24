#![no_std]
#![no_main]

use bsp::{
    entry,
    hal::{pio::PIOExt, Timer},
};
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_pio::Ws2812;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = rp_pico::XOSC_CRYSTAL_FREQ;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let (pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut pio = pio;
    let mut delay = delay;
    let mut ws = Ws2812::new(
        pins.gpio16.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    const NUM_LEDS: usize = 30;
    const BRIGHTNESS: u8 = 2;

    let mut data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];
    // Blink the LED's in a blue-green-red-white pattern.
    for led in data.iter_mut().step_by(3) {
        led.b = 255;
    }

    if NUM_LEDS > 1 {
        for led in data.iter_mut().skip(1).step_by(3) {
            led.g = 255;
        }
    }

    if NUM_LEDS > 2 {
        for led in data.iter_mut().skip(2).step_by(3) {
            led.r = 255;
        }
    }

    loop {
        for i in 0..(NUM_LEDS - 1) {
            ws.write(brightness(data.iter().cloned(), BRIGHTNESS))
                .unwrap();

            data.swap(i, i + 1);

            delay.delay_ms((1200 / NUM_LEDS).try_into().unwrap());
        }
    }
}
