use std::borrow::Borrow;
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LiquidCTLItem {
    pub bus: String,
    pub address: String,
    pub description: String,
    pub status: Vec<StatusItem>,
}

#[derive(Deserialize)]
pub struct StatusItem {
    pub key: String,
    pub unit: String,
    pub value: NumberOrString,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum NumberOrString {
  Number(f64),
  String(String),
}

pub fn get_liquidctl_data() -> (String, Vec<LiquidCTLItem>) {
    let out = Command::new("liquidctl").arg("status").arg("--json").output().unwrap();
    let out_str = String::from_utf8_lossy(&out.stdout);

    let result = serde_json::from_str(out_str.borrow());
    if let Err(e) = result {
        println!("{}", out_str);
        println!("{}", e);
        return (out_str.to_string(), Vec::new());
    }

    return (out_str.to_string(), result.unwrap());
}
