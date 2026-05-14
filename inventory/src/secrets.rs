#![allow(dead_code)] // wird ab S10 von Sync-Modulen verwendet

use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Entschluesselt eine sops-Datei und parsed sie als K=V env-Liste.
/// Schluesselmaterial wird via SOPS_AGE_KEY_FILE ans externe `sops` weitergereicht,
/// landet nie im Klartext auf Disk und nie in unserer Prozess-Env.
pub fn load(enc_path: &Path, age_key_path: &Path) -> Result<HashMap<String, String>> {
    if !enc_path.exists() {
        bail!("verschluesselte Datei fehlt: {}", enc_path.display());
    }
    if !age_key_path.exists() {
        bail!("age-Key fehlt: {}", age_key_path.display());
    }

    // --input-type dotenv noetig, weil sops die .enc-Extension nicht erkennt
    // und sonst JSON erwartet.
    let output = Command::new("sops")
        .arg("-d")
        .arg("--input-type")
        .arg("dotenv")
        .arg("--output-type")
        .arg("dotenv")
        .arg(enc_path)
        .env("SOPS_AGE_KEY_FILE", age_key_path)
        .output()
        .with_context(|| {
            format!(
                "sops -d {} (ist `sops` im PATH?)",
                enc_path.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "sops -d {} fehlgeschlagen ({}): {}",
            enc_path.display(),
            output.status,
            stderr.trim()
        );
    }

    let stdout =
        String::from_utf8(output.stdout).context("sops-Ausgabe ist kein UTF-8")?;
    parse_env(&stdout)
}

/// Einfacher dotenv-Parser. Keine Variablen-Expansion, keine Multiline-Strings.
fn parse_env(s: &str) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for (i, raw) in s.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (k, v) = line
            .split_once('=')
            .ok_or_else(|| anyhow!("Zeile {}: kein '=' in {:?}", i + 1, line))?;
        let key = k.trim();
        if key.is_empty() {
            bail!("Zeile {}: leerer Key", i + 1);
        }
        let value = strip_quotes(v.trim());
        map.insert(key.to_string(), value.to_string());
    }
    Ok(map)
}

fn strip_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tools_ready() -> bool {
        Command::new("sops").arg("--version").output().is_ok()
            && Command::new("age-keygen").arg("--version").output().is_ok()
    }

    /// Erzeugt ein age-Keypair im Tempdir und gibt (key_path, pub_string) zurueck.
    fn gen_age_key(dir: &Path) -> (std::path::PathBuf, String) {
        let key_path = dir.join("test.age");
        let out = Command::new("age-keygen")
            .arg("-o")
            .arg(&key_path)
            .output()
            .expect("age-keygen run");
        assert!(out.status.success(), "age-keygen failed: {}", String::from_utf8_lossy(&out.stderr));

        let pub_out = Command::new("age-keygen")
            .arg("-y")
            .arg(&key_path)
            .output()
            .expect("age-keygen -y run");
        assert!(pub_out.status.success(), "age-keygen -y failed");
        let pubkey = String::from_utf8(pub_out.stdout).unwrap().trim().to_string();
        (key_path, pubkey)
    }

    /// Schreibt plaintext und verschluesselt mit sops; gibt enc_path zurueck.
    fn encrypt_env(dir: &Path, pubkey: &str, content: &str) -> std::path::PathBuf {
        let plain = dir.join("plain.env");
        std::fs::write(&plain, content).unwrap();
        let enc = Command::new("sops")
            .arg("-e")
            .arg("--input-type")
            .arg("dotenv")
            .arg("--output-type")
            .arg("dotenv")
            .arg("--age")
            .arg(pubkey)
            .arg(&plain)
            .output()
            .expect("sops -e run");
        assert!(
            enc.status.success(),
            "sops -e failed: {}",
            String::from_utf8_lossy(&enc.stderr)
        );
        let enc_path = dir.join("plain.env.enc");
        std::fs::write(&enc_path, enc.stdout).unwrap();
        enc_path
    }

    #[test]
    fn parse_env_handles_quotes_and_comments() {
        let map = parse_env(
            "# comment\nHA_TOKEN=foo\nCCU_USER=\"with spaces\"\nEMPTY=\nQUOTED='abc'\n",
        )
        .unwrap();
        assert_eq!(map.get("HA_TOKEN"), Some(&"foo".to_string()));
        assert_eq!(map.get("CCU_USER"), Some(&"with spaces".to_string()));
        assert_eq!(map.get("EMPTY"), Some(&"".to_string()));
        assert_eq!(map.get("QUOTED"), Some(&"abc".to_string()));
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn parse_env_rejects_lines_without_equals() {
        let err = parse_env("HA_TOKEN\n").unwrap_err();
        assert!(format!("{err:#}").contains("kein '='"));
    }

    #[test]
    fn decrypts_with_correct_key() {
        if !tools_ready() {
            panic!("sops + age muessen im PATH sein fuer secrets-Tests");
        }
        let dir = tempfile::tempdir().unwrap();
        let (key, pubkey) = gen_age_key(dir.path());
        let enc = encrypt_env(
            dir.path(),
            &pubkey,
            "HA_TOKEN=foo\nCCU_USER=bar\nCCU_PASS=secret\n",
        );

        let map = load(&enc, &key).unwrap();
        assert_eq!(map.get("HA_TOKEN"), Some(&"foo".to_string()));
        assert_eq!(map.get("CCU_USER"), Some(&"bar".to_string()));
        assert_eq!(map.get("CCU_PASS"), Some(&"secret".to_string()));
    }

    #[test]
    fn fails_clearly_with_wrong_key() {
        if !tools_ready() {
            panic!("sops + age muessen im PATH sein");
        }
        let dir = tempfile::tempdir().unwrap();
        let (_correct_key, pubkey) = gen_age_key(dir.path());
        let other_dir = tempfile::tempdir().unwrap();
        let (wrong_key, _) = gen_age_key(other_dir.path());
        let enc = encrypt_env(dir.path(), &pubkey, "HA_TOKEN=foo\n");

        let err = load(&enc, &wrong_key).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("sops -d") && msg.contains("fehlgeschlagen"),
            "Fehlermeldung soll sops + fehlgeschlagen enthalten: {msg}"
        );
    }

    #[test]
    fn fails_clearly_when_enc_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let key = dir.path().join("nonexistent.age");
        std::fs::write(&key, "AGE-SECRET-KEY-1xxx").unwrap();
        let enc = dir.path().join("nonexistent.env.enc");

        let err = load(&enc, &key).unwrap_err();
        assert!(format!("{err:#}").contains("verschluesselte Datei fehlt"));
    }
}
