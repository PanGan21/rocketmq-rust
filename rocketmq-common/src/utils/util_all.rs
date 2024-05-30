/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Instant,
};

use chrono::{DateTime, Datelike, Local, TimeZone, Timelike, Utc};
use once_cell::sync::Lazy;
use tracing::{error, info};

use crate::common::mix_all::MULTI_PATH_SPLITTER;

const HEX_ARRAY: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

pub fn compute_elapsed_time_milliseconds(begin_time: Instant) -> u64 {
    let elapsed = begin_time.elapsed();
    elapsed.as_millis() as u64
}

pub fn is_it_time_to_do(when: &str) -> bool {
    let hours: Vec<&str> = when.split(";").collect();
    if !hours.is_empty() {
        let now = Local::now();
        for hour in hours {
            let now_hour: i32 = hour.parse().unwrap_or(0);
            if now_hour == now.hour() as i32 {
                return true;
            }
        }
    }
    false
}

pub fn time_millis_to_human_string2(t: i64) -> String {
    let dt = Utc.timestamp_millis_opt(t).unwrap();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02},{:03}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        dt.timestamp_subsec_millis(),
    )
}

pub fn time_millis_to_human_string3(t: i64) -> String {
    let dt = Utc.timestamp_millis_opt(t).unwrap();
    format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    )
}

pub fn time_millis_to_human_string(t: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp_millis(t);
    dt.as_ref().unwrap().format("%Y%m%d%H%M%S%3f").to_string()
}

pub fn is_path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn get_disk_partition_space_used_percent(path: &str) -> f64 {
    if path.is_empty() {
        error!(
            "Error when measuring disk space usage, path is null or empty, path: {}",
            path
        );
        return -1.0;
    }

    let path = Path::new(path);
    if !path.exists() {
        error!(
            "Error when measuring disk space usage, file doesn't exist on this path: {}",
            path.to_string_lossy()
        );
        return -1.0;
    }

    match fs::metadata(path) {
        Ok(metadata) => {
            let total_space = metadata.len();
            if total_space > 0 {
                match (fs::metadata(path), fs::metadata(path)) {
                    (Ok(metadata1), Ok(metadata2)) => {
                        let free_space = metadata1.len();
                        let usable_space = metadata2.len();
                        let used_space = total_space.saturating_sub(free_space);
                        let entire_space = used_space + usable_space;
                        let round_num = if used_space * 100 % entire_space != 0 {
                            1
                        } else {
                            0
                        };
                        let result = used_space * 100 / entire_space + round_num;
                        return result as f64 / 100.0;
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error!(
                            "Error when measuring disk space usage, got exception: {:?}",
                            e
                        );
                        return -1.0;
                    }
                }
            }
        }
        Err(e) => {
            error!(
                "Error when measuring disk space usage, got exception: {:?}",
                e
            );
            return -1.0;
        }
    }

    -1.0
}

pub fn bytes_to_string(src: &[u8]) -> String {
    let mut hex_chars = Vec::with_capacity(src.len() * 2);
    for &byte in src {
        let v = byte as usize;
        hex_chars.push(HEX_ARRAY[v >> 4]);
        hex_chars.push(HEX_ARRAY[v & 0x0F]);
    }
    hex_chars.into_iter().collect()
}

pub fn write_int(buffer: &mut [char], pos: usize, value: i32) {
    let value_str = format!("{:X}", value);
    let value_chars: Vec<char> = value_str.chars().collect();
    for (i, &c) in value_chars.iter().enumerate() {
        buffer[pos + i] = c;
    }
}

pub fn write_short(buffer: &mut [char], pos: usize, value: i16) {
    let value_str = format!("{:X}", value);
    let value_chars: Vec<char> = value_str.chars().collect();
    for (i, &c) in value_chars.iter().enumerate() {
        buffer[pos + i] = c;
    }
}

fn string_to_bytes(hex_string: impl Into<String>) -> Option<Vec<u8>> {
    let hex_string = hex_string.into();
    if hex_string.is_empty() {
        return None;
    }

    let hex_string = hex_string.to_uppercase();
    let length = hex_string.len() / 2;
    let mut bytes = Vec::<u8>::with_capacity(length);

    for i in 0..length {
        let pos = i * 2;
        let byte = char_to_byte(hex_string.chars().nth(pos)?) << 4
            | char_to_byte(hex_string.chars().nth(pos + 1)?);

        bytes.push(byte);
    }

    Some(bytes)
}

fn char_to_byte(c: char) -> u8 {
    let hex_chars = "0123456789ABCDEF";
    hex_chars.find(c).unwrap_or(0) as u8
}

pub fn offset_to_file_name(offset: u64) -> String {
    format!("{:020}", offset)
}

pub fn ensure_dir_ok(dir_name: &str) {
    if !dir_name.is_empty() {
        let multi_path_splitter = MULTI_PATH_SPLITTER.as_str();
        if dir_name.contains(multi_path_splitter) {
            for dir in dir_name.trim().split(&multi_path_splitter) {
                create_dir_if_not_exist(dir);
            }
        } else {
            create_dir_if_not_exist(dir_name);
        }
    }
}

fn create_dir_if_not_exist(dir_name: &str) {
    let path = Path::new(dir_name);
    if !path.exists() {
        match fs::create_dir_all(path) {
            Ok(_) => info!("{} mkdir OK", dir_name),
            Err(_) => info!("{} mkdir Failed", dir_name),
        }
    }
}
#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn compute_elapsed_time_milliseconds_returns_correct_duration() {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let elapsed = compute_elapsed_time_milliseconds(start);
        assert!(elapsed >= 100);
    }

    #[test]
    fn is_it_time_to_do_returns_true_when_current_hour_is_in_input() {
        let current_hour = Local::now().hour();
        assert_eq!(is_it_time_to_do(&current_hour.to_string()), true);
    }

    #[test]
    fn is_it_time_to_do_returns_false_when_current_hour_is_not_in_input() {
        let current_hour = (Local::now().hour() + 1) % 24;
        assert_eq!(is_it_time_to_do(&current_hour.to_string()), false);
    }

    #[test]
    fn time_millis_to_human_string_formats_correctly() {
        let timestamp = 1625140800000; // 2021-07-01T12:00:00Z
        assert_eq!(time_millis_to_human_string(timestamp), "20210701120000000");
    }

    #[test]
    fn is_path_exists_returns_true_for_existing_path() {
        assert_eq!(is_path_exists("."), true);
    }

    #[test]
    fn is_path_exists_returns_false_for_non_existing_path() {
        assert_eq!(is_path_exists("./non_existing_path"), false);
    }

    #[test]
    fn bytes_to_string_converts_correctly() {
        let bytes = [0x41, 0x42, 0x43];
        assert_eq!(bytes_to_string(&bytes), "414243");
    }

    #[test]
    fn offset_to_file_name_formats_correctly() {
        assert_eq!(offset_to_file_name(123), "00000000000000000123");
    }

    #[test]
    fn ensure_dir_ok_creates_directory_if_not_exists() {
        let dir_name = "./test_dir";
        ensure_dir_ok(dir_name);
        assert_eq!(is_path_exists(dir_name), true);
        std::fs::remove_dir(dir_name).unwrap();
    }
}
