#![no_std]
#![no_main]

use bsp::{
    entry,
    hal::{pio::PIOExt, Timer},
};
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rand::SeedableRng;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use rand::prelude::*;
use smart_leds::{SmartLedsWrite, RGB8};
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
        pins.b_power_save.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    let mut color = [0_u8, 0, 0];
    let mut muls = [1_i8, 1, 1];
    let step = 5_i8;

    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    loop {
        for (color, mul) in color.iter_mut().zip(muls.iter_mut()) {
            if rng.gen_bool(0.5) {
                match color.checked_add_signed(step * *mul) {
                    Some(x) => *color = x,
                    None => {
                        *mul *= -1;
                        *color = color.checked_add_signed(step * *mul).unwrap();
                    }
                };

                break;
            };
        }
        let color = RGB8 {
            r: color[0],
            g: color[1],
            b: color[2],
        };
        ws.write([color].iter().copied()).unwrap();

        // info!("on!");
        // led_pin.set_high().unwrap();
        // delay.delay_ms(500);
        // info!("off!");
        // led_pin.set_low().unwrap();
        delay.delay_ms(50);
    }
}
