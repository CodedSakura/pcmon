use std::process::Command;

pub struct NvidiaCard {
    pub name: String,
    pub temperature: u8,
    pub mem_used: u16,
    pub mem_total: u16,
    pub utilization: f32,
    pub fan_speed: i16,
}

pub fn get_nvidia_data() -> Vec<NvidiaCard> {
    let cmd_out = Command::new("nvidia-smi")
        .arg("--query-gpu=name,temperature.gpu,memory.used,memory.total,utilization.gpu,fan.speed")
        .arg("--format=csv,noheader,nounits")
        .output()
        .unwrap();
    let cmd_str = String::from_utf8_lossy(&cmd_out.stdout);

    cmd_str.lines().map( |x| {
        let args = x.split(", ").collect::<Vec<&str>>();
        NvidiaCard {
            name: args[0].to_string(),
            temperature: args[1].parse().unwrap(),
            mem_used: args[2].parse().unwrap(),
            mem_total: args[3].parse().unwrap(),
            utilization: args[4].parse().unwrap(),
            fan_speed: args[5].parse().unwrap(),
        }
    }).collect()
}
