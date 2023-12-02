#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::PwmPin;
use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    // let external_xtal_freq_hz = 1_000_000u32; // 1 MHz
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

    let pwm_slices = bsp::hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let mut slices = pwm_slices;
    let mut delay = delay;
    let pwm = &mut slices.pwm1;
    let gen = FreqPWM::new(DIV);
    let c = [16_u16, 33, 65, 131, 262, 523, 1046, 2093];
    let d = [18, 37, 73, 147, 294, 587, 1174, 2349];
    let e = [21, 41, 82, 165, 330, 660, 1318, 2637];
    let f = [22, 44, 87, 175, 350, 700, 1396, 2793];
    let g = [25, 49, 98, 196, 392, 784, 1567, 3135];
    let a_s = [29, 58, 116, 233, 466, 932, 1864, 3729];
    let a = [27, 55, 110, 220, 440, 880, 1760, 3520];
    let b = [31, 62, 123, 247, 494, 988, 1975, 3951];

    #[rustfmt::skip]
        #[allow(unused)]
        let jingle_bells = [
            e[5], e[5], e[5], 0, 
            e[5], e[5], e[5], 0,
            e[5], g[5], c[5], d[5], e[5], 0, 
            f[5], f[5], f[5], f[5], f[5], e[5], e[5], e[5], e[5], d[5], d[5], e[5], d[5], 0, g[5], 0,
            e[5], e[5], e[5], 0,
            e[5], e[5], e[5], 0,
            e[5], g[5], c[5], d[5], e[5], 0, 
            f[5], f[5], f[5], f[5], f[5], e[5], e[5], e[5], g[5], g[5], f[5], d[5], c[5], 0,
        ];
    #[rustfmt::skip]
        #[allow(unused)]
        let mario_main = [
            e[5], e[5], 0, e[5],
            0, c[5], e[5], 0,
            g[5], 0, 0, 0,
            g[4], 0, 0, 0,

            c[5], 0, 0, g[4],
            0, 0, e[4], 0,
            0, a[4], 0, b[4],
            0, a_s[4], a[4], 0,

            g[4], e[5], g[5],
            a[5], 0, f[5], g[5],
            0, e[5], 0, c[5],
            d[5], b[4], 0, 0,

            c[5], 0, 0, g[5],
            0, 0, e[4], 0,
            0, a[4], 0, b[4],
            0, a_s[4], a[4],0,

            g[4], e[5], g[5],
            a[5], 0, f[5], g[5], 
            0, e[5], 0, c[5],
            d[5], b[5], 0, 0,
        ];
    let mario_main_tempo = [
        12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
        12, 12, 12, 12, 12, 12, 12, 12, 12, 9, 9, 9, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
        12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 9, 9, 9, 12, 12, 12,
        12, 12, 12, 12, 12, 12, 12, 12, 12,
    ];

    const DIV: u8 = 40;
    pwm.set_div_int(DIV);
    pwm.enable();
    pwm.channel_a.output_to(pins.gpio2);
    let tempo: u32 = 1000;
    let button = pins.vbus_detect.into_pull_up_input();
    info!("waiting for button press");
    loop {
        if button.is_high().unwrap() {
            continue;
        }
        for (freq, ratio) in mario_main
            .iter()
            .copied()
            .zip(mario_main_tempo.iter().copied())
        {
            // for (freq, ratio) in jingle_bells.iter().copied().zip(core::iter::repeat(11_u32)) {
            let note_duration = tempo / ratio as u32;
            let top = gen.freq_to_top(freq as u32);
            info!(
                "freq = {}, top = {}, note duration = {}",
                freq, top, note_duration
            );

            pwm.set_top(top);
            pwm.set_counter(0);
            pwm.channel_a.set_duty(top / 2);

            pwm.channel_a.enable();
            delay.delay_ms(note_duration);
            pwm.channel_a.disable();

            delay.delay_ms((note_duration as f32 * 1.3) as u32);
        }
        info!("waiting for button press");
    }
}

struct FreqPWM {
    div_int: u8,
}

impl FreqPWM {
    fn new(div_int: u8) -> Self {
        Self { div_int }
    }

    fn freq_to_top(&self, freq: u32) -> u16 {
        if freq == 0 {
            return 0;
        }

        let div = self.div_int as f32;
        let freq = freq as f32;
        let top = bsp::XOSC_CRYSTAL_FREQ as f32 / div / freq;
        top as u16
    }
}
