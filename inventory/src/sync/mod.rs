pub mod ccu;
pub mod ha;
pub mod hue;
pub mod shelly;

// WIP — Node-RED integration, branch `feat/nodered-integration` only.
// Intentionally compiled but NOT wired into the CLI `sync` dispatch.
// See docs/nodered-integration.md and stack-master ADR-0011.
#[allow(dead_code)]
pub mod nodered;
