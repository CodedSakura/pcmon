use std::ops::Sub;
use std::thread;
use std::time::{Duration, SystemTime};

use influx_db_client::{Point, Points, Precision};
use libmedium::{parse_hwmons, sensors::{Input, Sensor}};
use sysinfo::{ComponentExt, NetworkExt, ProcessorExt, System, SystemExt};
use tokio;

use crate::diskstats::get_diskstats_data;
use crate::liquidctl::{get_liquidctl_data, NumberOrString};
use crate::nvidia_smi::get_nvidia_data;

mod liquidctl;
mod nvidia_smi;
mod diskstats;

#[tokio::main]
async fn main() {
    let mut influx = influx_db_client::Client::default().set_authentication("root", "root");

    let mut sys = System::new_all();

    influx.create_user("grafana", "grafanaPass", false).await.unwrap();
    influx.create_database("test").await.unwrap();
    influx.switch_database("test");

    loop {
        let top_time = SystemTime::now();

        let mut points: Vec<Point> = Vec::new();

        let (_, liquid_dev) = get_liquidctl_data();
        for device in liquid_dev {
            let mut point = Point::new("liquidctl")
                .add_tag("device", device.description)
                .add_tag("bus", device.bus)
                .add_tag("address", device.address);
            for x in device.status {
                point = match x.value {
                    NumberOrString::Number(num) => point.add_field(x.key, num),
                    NumberOrString::String(str) => point.add_field(x.key, str),
                };
            }
            points.push(point)
        };

        for disk in get_diskstats_data() {
            points.push(
                Point::new("diskstats")
                    .add_tag("disk_name", disk.name)
                    .add_field("time_io_ms", disk.time_io_ms as i64)
                    .add_field("wighted_time_io_ms", disk.weighted_time_io_ms as i64)
                    .add_field("time_writing_ms", disk.time_writing_ms as i64)
                    .add_field("time_reading_ms", disk.time_reading_ms as i64)
            );
        }

        for x in get_nvidia_data() {
            points.push(
                Point::new("nvidia")
                    .add_tag("device", x.name)
                    .add_field("temperature", x.temperature as i64)
                    .add_field("fan_speed", x.fan_speed as i64)
                    .add_field("utilization", x.utilization as f64)
                    .add_field("mem_used", x.mem_used as i64)
                    .add_field("mem_total", x.mem_total as i64)
            );
        }

        match parse_hwmons() {
            Ok(hwmons) => {
                for (_hw_id, hw_name, hw) in &hwmons {
                    for (_, fan) in hw.fans() {
                        points.push(
                            Point::new("hwmon")
                                .add_tag("category", "fans")
                                .add_tag("hw_name", hw_name)
                                .add_tag("fan", fan.name())
                                .add_field("speed", fan.read_input().unwrap().as_rpm() as i64)
                        );
                    }
                    for (_, temp) in hw.temps() {
                        points.push(
                            Point::new("hwmon")
                                .add_tag("category", "temps")
                                .add_tag("hw_name", hw_name)
                                .add_tag("temp", temp.name())
                                .add_field("celsius", temp.read_input().unwrap().as_degrees_celsius())
                        )
                    }
                    // for (_, pwm) in hw.writeable_pwms() {
                    //     println!("{}", pwm.name());
                    // }
                }
            },
            Err(e) => {
                println!("HWMons failed:");
                println!("{}", e);
            }
        };

        sys.refresh_all();
        for x in sys.components() {
            points.push(
                Point::new("sysinfo")
                    .add_tag("type", "components")
                    .add_tag("label", x.label())
                    .add_field("temperature", x.temperature() as f64)
            );
        }
        for x in sys.processors() {
            points.push(
                Point::new("sysinfo")
                    .add_tag("type", "processors")
                    .add_tag("name", x.name())
                    .add_tag("brand", x.brand())
                    .add_field("usage", x.cpu_usage() as f64)
                    .add_field("frequency", x.frequency() as i64)
            );
        }
        let load_avg = sys.load_average();
        points.push(
            Point::new("sysinfo")
                .add_tag("type", "load_averages")
                .add_field("one", load_avg.one)
                .add_field("five", load_avg.five)
                .add_field("fifteen", load_avg.fifteen)
        );
        points.push(
            Point::new("sysinfo")
                .add_tag("type", "memory")
                .add_field("used_memory", sys.used_memory() as i64)
                .add_field("total_memory", sys.total_memory() as i64)
                .add_field("used_swap", sys.used_swap() as i64)
                .add_field("total_swap", sys.total_swap() as i64)
        );

        for (name, data) in sys.networks() {
            points.push(
                Point::new("sysinfo")
                    .add_tag("type", "network")
                    .add_tag("interface", name.as_str())
                    .add_field("received", data.received() as i64)
                    .add_field("transmitted", data.transmitted() as i64)
            );
        }

        match influx.clone().write_points(Points::create_new(points), Some(Precision::Seconds), None).await {
            Ok(_) => {}
            Err(e) => {
                println!("influx write failed:");
                println!("{}", e);
            }
        };

        // sleep 5 seconds minus time difference between start of loop and now
        thread::sleep(
            Duration::from_secs(5)
                .sub(SystemTime::now().duration_since(top_time).unwrap())
        );
    }
}
