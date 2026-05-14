#![allow(dead_code)] // wird ab S9 / S12 verwendet
use crate::types::Device;
use anyhow::{Context, Result};
use std::path::Path;

pub fn load_devices(path: impl AsRef<Path>) -> Result<Vec<Device>> {
    let path = path.as_ref();
    let s = std::fs::read_to_string(path)
        .with_context(|| format!("lesen {}", path.display()))?;
    let devices: Vec<Device> = serde_yaml_ng::from_str(&s)
        .with_context(|| format!("yaml parsen {}", path.display()))?;
    Ok(devices)
}

pub fn save_devices(path: impl AsRef<Path>, devices: &[Device]) -> Result<()> {
    let path = path.as_ref();
    let s = serde_yaml_ng::to_string(devices)?;
    std::fs::write(path, s).with_context(|| format!("schreiben {}", path.display()))?;
    Ok(())
}
