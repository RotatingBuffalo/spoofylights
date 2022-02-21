use byteorder::{ReadBytesExt, BE};
use spoofylightslib::frame::{algos::Algos, pixel::Pixel, Frame};
#[cfg(target_arch = "arm")]
use spoofylightslib::raymond::hardware::Hardware;
use spoofylightslib::raymond::javasimulator::JavaSimulator;
use spoofylightslib::raymond::Raymond;
use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
fn cava_setup(num_bars: u8, smoothing: u8, framerate: u8, sensitivity: u8) {
    let config = format!(
        "
[general]
bars = {}
framerate = {}
sensitivity = {}
[output]
method = raw
raw_target = \"/dev/stdout\"
bit_format = \"16bit\"
[smoothing]
integral = {}
",
        num_bars, framerate, sensitivity, smoothing,
    );
    fs::write("cavaconf", config).ok();
}
fn main() {
    const NUM_BARS: u8 = 32;
    const SMOOTHING: u8 = 77;
    const FRAMERATE: u8 = 15;
    const SENSITIVITY: u8 = 3;
    {
        cava_setup(NUM_BARS, SMOOTHING, FRAMERATE, SENSITIVITY)
    }
    #[cfg(target_arch = "arm")]
    {
        // interrupt handler, so the matrix doesn't have
        // residual garbage left over on it.
        ctrlc::set_handler(|| {
            panic!();
        })
        .expect("Error setting interrupt handler?");

        // cava stuff.
        let mut cava = Command::new("cava")
            .arg("-p")
            .arg("cavaconf")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to run cava. Is it installed?");
        let mut bar_vals: [u16; NUM_BARS as usize] = [0; NUM_BARS as usize];
        let mut stdout = cava.stdout.take().unwrap();

        // rgb matrix display stuff.
        let mut f = Frame::new(Algos::hue_wave);
        let mut target = Hardware::new();
        target.connect();
        // program loop
        let mut life: i32 = 0;
        loop {
            stdout
                .read_u16_into::<BE>(&mut bar_vals)
                .expect("something has gone awry with byteorder.");

            for i in 0..bar_vals.len() {
                bar_vals[i] = bar_vals[i] / 2048;
            }
            for i in 0..32 {
                for j in 0..(32 - bar_vals[i / (32 / NUM_BARS) as usize]) {
                    f.this[(j as usize, i as usize)] = Pixel::new(Some((0, 0, 0)));
                }
            }
            target.send_frame(&mut f);
            Frame::tick(&mut f, life);
            life = life + 1;
            if life == i32::MAX {
                life = 0;
            }
        }
    }
    #[cfg(target_arch = "x86_64")]
    {
        // cava stuff.
        let mut cava = Command::new("cava")
            .arg("-p")
            .arg("cavaconf")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to run cava. Is it installed?");
        let mut bar_vals: [u16; NUM_BARS as usize] = [0; NUM_BARS as usize];
        let mut stdout = cava.stdout.take().unwrap();

        // rgb matrix display stuff.
        let mut f = Frame::new(Algos::hue_wave);
        let mut target = JavaSimulator::new();
        target.connect();
        // program loop
        let mut life: i32 = 0;
        loop {
            stdout
                .read_u16_into::<BE>(&mut bar_vals)
                .expect("something has gonw awry with byteorder.");

            for i in 0..bar_vals.len() {
                bar_vals[i] = bar_vals[i] / 2048;
            }
            println!("{:?}", bar_vals);
            for i in 0..32 {
                for j in 0..(32 - bar_vals[i / (32 / NUM_BARS) as usize]) {
                    f.this[(j as usize, i as usize)] = Pixel::new(Some((0, 0, 0)));
                }
            }
            target.send_frame(&mut f);
            thread::sleep(Duration::new(0, 40000000));
            Frame::tick(&mut f, life);
            life = life + 1;
            if life == i32::MAX {
                life = 0;
            }
        }
    }
}
