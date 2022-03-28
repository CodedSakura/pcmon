use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct IOStats {
    pub maj_num: u16,
    pub min_num: u16,
    pub name: String,
    pub reads_comp: u32,
    pub reads_merged: u32,
    pub sectors_read: u32,
    pub time_reading_ms: u32,
    pub writes_comp: u32,
    pub writes_merged: u32,
    pub sectors_written: u32,
    pub time_writing_ms: u32,
    pub io_in_progress: u32,
    pub time_io_ms: u32,
    pub weighted_time_io_ms: u32,
}

// https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats
pub fn get_diskstats_data() -> Vec<IOStats> {
    let file = File::open("/proc/diskstats").unwrap();
    let reader = BufReader::new(file);

    reader.lines()
        .map( |res_line| {
            let line = res_line.unwrap();
            let data = line
                .split(" ")
                .filter(|d| d.len() > 0)
                .collect::<Vec<&str>>();
            IOStats {
                maj_num: data[0].parse().unwrap(),
                min_num: data[1].parse().unwrap(),
                name: data[2].to_string(),
                reads_comp: data[3].parse().unwrap(),
                reads_merged: data[4].parse().unwrap(),
                sectors_read: data[5].parse().unwrap(),
                time_reading_ms: data[6].parse().unwrap(),
                writes_comp: data[7].parse().unwrap(),
                writes_merged: data[8].parse().unwrap(),
                sectors_written: data[9].parse().unwrap(),
                time_writing_ms: data[10].parse().unwrap(),
                io_in_progress: data[11].parse().unwrap(),
                time_io_ms: data[12].parse().unwrap(),
                weighted_time_io_ms: data[13].parse().unwrap(),
            }
        } )
        .collect()
}
