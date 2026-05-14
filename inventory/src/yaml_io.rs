use crate::types::Device;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn load_devices(path: impl AsRef<Path>) -> Result<Vec<Device>> {
    let path = path.as_ref();
    let s = std::fs::read_to_string(path)
        .with_context(|| format!("lesen {}", path.display()))?;
    let devices: Vec<Device> = serde_yaml_ng::from_str(&s)
        .with_context(|| format!("yaml parsen {}", path.display()))?;
    Ok(devices)
}

#[allow(dead_code)] // wird in S12b vom git-publish-Pfad genutzt
pub fn save_devices(path: impl AsRef<Path>, devices: &[Device]) -> Result<()> {
    let path = path.as_ref();
    let s = serde_yaml_ng::to_string(devices)?;
    std::fs::write(path, s).with_context(|| format!("schreiben {}", path.display()))?;
    Ok(())
}

/// Schreibt die Geraete einer Source deterministisch nach <dir>/<source>.yaml.
/// "Deterministisch" heisst: Input wird nach source_id sortiert, sodass identische
/// Eingaben byte-identische Ausgaben ergeben — damit git nur bei echten Datenaenderungen
/// einen Diff sieht.
pub fn write_devices_for_source(
    dir: impl AsRef<Path>,
    source: &str,
    devices: &[Device],
) -> Result<PathBuf> {
    let dir = dir.as_ref();
    std::fs::create_dir_all(dir).with_context(|| format!("mkdir {}", dir.display()))?;
    let mut sorted: Vec<Device> = devices
        .iter()
        .filter(|d| d.source == source)
        .cloned()
        .collect();
    sorted.sort_by(|a, b| a.source_id.cmp(&b.source_id));
    let path = dir.join(format!("{source}.yaml"));
    let s = serde_yaml_ng::to_string(&sorted)?;
    std::fs::write(&path, s)
        .with_context(|| format!("schreiben {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_device(source: &str, id: &str, name: &str) -> Device {
        Device {
            source: source.into(),
            source_id: id.into(),
            name: name.into(),
            manufacturer: None,
            model: None,
            kind: None,
            room: None,
        }
    }

    #[test]
    fn write_output_is_byte_identical_for_same_input() {
        let dir = tempfile::tempdir().unwrap();
        let devs = vec![
            make_device("ha", "light.b", "B"),
            make_device("ha", "light.a", "A"),
        ];
        let p1 = write_devices_for_source(dir.path(), "ha", &devs).unwrap();
        let bytes1 = std::fs::read(&p1).unwrap();
        let p2 = write_devices_for_source(dir.path(), "ha", &devs).unwrap();
        let bytes2 = std::fs::read(&p2).unwrap();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn write_sorts_by_source_id_regardless_of_input_order() {
        let dir = tempfile::tempdir().unwrap();
        let devs_a = vec![
            make_device("ha", "light.b", "B"),
            make_device("ha", "light.a", "A"),
            make_device("ha", "light.c", "C"),
        ];
        let mut devs_b = devs_a.clone();
        devs_b.reverse();
        let p1 = write_devices_for_source(dir.path().join("a"), "ha", &devs_a).unwrap();
        let p2 = write_devices_for_source(dir.path().join("b"), "ha", &devs_b).unwrap();
        assert_eq!(std::fs::read(&p1).unwrap(), std::fs::read(&p2).unwrap());
    }

    #[test]
    fn write_filters_to_requested_source_only() {
        let dir = tempfile::tempdir().unwrap();
        let devs = vec![
            make_device("ha", "light.a", "A"),
            make_device("ccu", "ABC123", "Therm"),
            make_device("ha", "light.b", "B"),
        ];
        let p = write_devices_for_source(dir.path(), "ha", &devs).unwrap();
        let content = std::fs::read_to_string(&p).unwrap();
        let parsed: Vec<Device> = serde_yaml_ng::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.iter().all(|d| d.source == "ha"));
    }

    #[test]
    fn write_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        let devs = vec![make_device("ha", "light.x", "X")];
        let p = write_devices_for_source(&nested, "ha", &devs).unwrap();
        assert!(p.exists());
    }
}
