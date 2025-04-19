use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

const EC_IO_FILE: &str = "/sys/kernel/debug/ec/ec0/io";
const FAN1_OFFSET: u64 = 0x34; // Fan 1 Speed Set (units of 100RPM)
const FAN2_OFFSET: u64 = 0x35; // Fan 2 Speed Set (units of 100RPM)
const CPU_TEMP_OFFSET: u64 = 0x57; // CPU Temp (°C)
const GPU_TEMP_OFFSET: u64 = 0xB7; // GPU Temp (°C)
const BIOS_CONTROL_OFFSET: u64 = 0x62; // BIOS Control
const FAN1_MAX: u8 = 55; // Max speed for Fan 1
const FAN2_MAX: u8 = 57; // Max speed for Fan 2

fn read_ec_register(offset: u64) -> u8 {
    let mut file = File::open(EC_IO_FILE).expect("Failed to open EC IO file. Ensure you have the necessary permissions.");
    file.seek(SeekFrom::Start(offset))
        .expect("Failed to seek to EC register.");
    let mut buffer = [0u8; 1];
    file.read_exact(&mut buffer)
        .expect("Failed to read EC register.");
    buffer[0]
}

fn write_ec_register(offset: u64, value: u8) {
    let mut file = OpenOptions::new()
        .write(true)
        .open(EC_IO_FILE)
        .expect("Failed to open EC IO file. Ensure you have the necessary permissions.");
    file.seek(SeekFrom::Start(offset))
        .expect("Failed to seek to EC register.");
    file.write_all(&[value])
        .expect("Failed to write to EC register.");
}

fn get_max_temp() -> u8 {
    let cpu_temp = read_ec_register(CPU_TEMP_OFFSET);
    let gpu_temp = read_ec_register(GPU_TEMP_OFFSET);
    cpu_temp.max(gpu_temp)
}

fn set_fan_speed(fan1_speed: u8, fan2_speed: u8) {
    write_ec_register(FAN1_OFFSET, fan1_speed);
    write_ec_register(FAN2_OFFSET, fan2_speed);
}

fn disable_bios_control() {
    write_ec_register(BIOS_CONTROL_OFFSET, 0x06); // Disable BIOS control
}

// fn enable_bios_control() {
//    write_ec_register(BIOS_CONTROL_OFFSET, 0x00); // Enable BIOS control
// }

fn main() {
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("Root access is required to run this program.");
        exit(1);
    }

    disable_bios_control();

    let temp_curve = [45, 55, 60, 70, 75, 80, 85, 93];
    let speed_curve = [0, 20, 30, 60, 70, 80, 90, 100];
    let idle_speed = 0;
    let poll_interval = Duration::from_secs(1);

    let mut previous_speed = (0, 0);

    loop {
        let temp = get_max_temp();

        let speed = if temp <= temp_curve[0] {
            idle_speed
        } else if temp >= temp_curve[temp_curve.len() - 1] {
            speed_curve[speed_curve.len() - 1]
        } else {
            let index = temp_curve.iter().position(|&t| t > temp).unwrap();
            let t0 = temp_curve[index - 1];
            let t1 = temp_curve[index];
            let s0 = speed_curve[index - 1];
            let s1 = speed_curve[index];
            (s0 as usize + ((s1 - s0) as usize * (temp - t0) as usize / (t1 - t0) as usize)) as u8
        };

        let fan1_speed = ((FAN1_MAX as u16 * speed as u16) / 100) as u8;
        let fan2_speed = ((FAN2_MAX as u16 * speed as u16) / 100) as u8;

        if previous_speed != (fan1_speed, fan2_speed) {
            set_fan_speed(fan1_speed, fan2_speed);
            previous_speed = (fan1_speed, fan2_speed);
        }

        sleep(poll_interval);
    }
}