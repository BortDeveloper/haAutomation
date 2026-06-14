# Getting Started — First Run on Home Network

> **PoC / Experimental**: see [README.md](../README.md) for the alpha
> disclaimer. The inventory backend has not undergone independent
> security review or long-term operational validation. Treat every
> step in this guide as best-effort hardening for a controlled
> home-network experiment, not a production deployment recipe.

This walkthrough takes a freshly cloned `haAutomation` repo and runs
the inventory backend once against a real home network. The target
setup is:

- a Linux build host reachable over Tailscale (or SSH tunnel) that
  hosts the build artifacts and the inventory database,
- a Home Assistant instance on the home LAN,
- a Homematic CCU3 / RaspberryMatic on the home LAN,
- optional Philips Hue bridge(s),
- optional Shelly devices reachable on the same LAN segment as the
  build host (mDNS-discoverable).

All security mitigations in this guide map to findings in the
internal security audit dated 2026-05-20 (CRITICAL/HIGH items only;
the MEDIUM/LOW items are tracked as follow-up issues).

## Prerequisites

- Linux build host reachable over Tailscale (or SSH tunnel)
- Rust 1.95 (matches `home-inventory/rust-toolchain.toml` / `Cargo.toml`
  `rust-version`)
- Home Assistant with admin access (for a Long-Lived Access Token)
- Optional: Homematic CCU / RaspberryMatic with XML-API addon enabled
- Optional: Philips Hue Bridge(s) (HTTP REST API v1)
- Optional: Shelly devices on the same LAN segment as the build host
  (mDNS announcements must reach the build host)

## Phase 0 — Build host setup

> **Security note (R-HIGH-2)**: SSH access to the build host should
> use an Ed25519 key, `PermitRootLogin prohibit-password`, and
> `PasswordAuthentication no` in `/etc/ssh/sshd_config`. Tailscale
> ACLs should restrict the build host to the operator's device(s)
> only; do not rely on the tailnet boundary as your only auth
> layer (Tailscale auth-key leaks have happened in the wild — see
> NIST SP 800-207 "Zero Trust" § 3).

Install build dependencies and pin the Rust toolchain:

```bash
apt update
apt install -y build-essential pkg-config libssl-dev libsqlite3-dev git curl

# rustup installer — TLS-pinned download
curl --proto '=https' --tlsv1.2 -sSf -o /tmp/rustup-init.sh https://sh.rustup.rs
# Optional but recommended: inspect the script before running it
less /tmp/rustup-init.sh
sh /tmp/rustup-init.sh -y --default-toolchain 1.95
rm /tmp/rustup-init.sh
source "$HOME/.cargo/env"
```

Clone the repo and run a locked release build:

```bash
git clone https://github.com/BortDeveloper/haAutomation.git ~/haAutomation
cd ~/haAutomation/home-inventory

# Optional: dependency-advisory check before build
cargo install cargo-audit --locked
cargo audit         # stop on findings, update Cargo.lock, then rebuild

cargo build --release --locked
./target/release/home-inventory --help
```

`--locked` ensures `Cargo.lock` is honored byte-for-byte; do not
bypass it.

## Phase 1 — Credentials

> **Security note**: Treat every token in this section as a secret.
> Do not paste them into chat, do not commit them to git, and do not
> pass them as CLI arguments (see R-CRIT-1 below).

| Source | What you need |
|---|---|
| Home Assistant | A Long-Lived Access Token from your HA user profile (`/profile` → "Long-Lived Access Tokens"). |
| Homematic CCU | The CCU's LAN URL, e.g. `http://ccu.example.local`. If your RaspberryMatic has "Authentication" enabled (default since RaspberryMatic 3.65+), you also need a username (default: `Admin`) and the matching password. Pass them via the `CCU_USER` / `CCU_PASSWORD` env vars, not on the command line. |
| Philips Hue (optional) | One API key per bridge. Pair via the bridge's link button + `POST http://<bridge-ip>/api -d '{"devicetype":"haAutomation"}'`. |
| Shelly (optional) | None — mDNS-discovered at sync time. |

> **HA Long-Lived Tokens** do not expire (HA default: 10 years).
> They grant full API access to the HA instance. Treat them like
> root passwords.

## Phase 2 — Configuration (Hue is optional)

The Hue config is the only on-disk credential file the inventory
backend reads directly. It lives outside the git tree:

```bash
# Restrict the default umask first, so any new file is created 600
umask 077

mkdir -p ~/haAutomation/home-inventory/local
cat > ~/haAutomation/home-inventory/local/hue.yml <<'EOF'
- ip: 192.168.x.x
  token: <bridge-1-api-key>
  name: bridge-1
EOF

# Belt-and-braces: enforce the mode explicitly (R-HIGH-1)
chmod 600 ~/haAutomation/home-inventory/local/hue.yml
ls -l ~/haAutomation/home-inventory/local/hue.yml  # expect: -rw-------
```

If you do not have any Hue bridges, **skip this phase entirely**.
`sync hue` is optional and exits cleanly without a config (see
Phase 3).

## Phase 3 — Sync run (security-hardened)

Tokens are passed via environment, never as `--token <value>`.
Argv values land in `/proc/<pid>/cmdline`, `ps -ef`, and
`~/.bash_history`; the latter persists across reboots and is the
realistic exfiltration path for a single-user host (R-CRIT-1; OWASP
ASVS V2.10.4; CWE-214).

```bash
# Tokens via env, NOT via CLI argv
export HA_TOKEN='<long-lived-token>'
export HA_URL='http://ha.example.local:8123'  # replace with your own Home Assistant URL
export INVENTORY_DB=~/haAutomation/local/inventory.db
export INVENTORY_YAML_DIR=~/haAutomation/local/yaml

# CRITICAL guard rail: make sure publish mode is OFF for the first run
# (it is off by default, but we make it explicit — see R-HIGH-3).
unset INVENTORY_PUBLISH
unset INVENTORY_PUBLISH_CONFIRM
env | grep -i 'INVENTORY_PUBLISH' && {
  echo "WARNING: INVENTORY_PUBLISH variable still set in env — unset and retry."
  exit 1
}

# Initialize the DB
./target/release/home-inventory migrate

# Lock down the DB file (R-MED-3)
chmod 600 "$INVENTORY_DB"
mkdir -p "$INVENTORY_YAML_DIR"
chmod 700 "$INVENTORY_YAML_DIR"

# Sync each source (Hue is optional — omit --config to skip)
./target/release/home-inventory sync ha          # uses HA_URL + HA_TOKEN from env

# CCU (Homematic) — with Basic Auth if your RaspberryMatic has
# "Authentication" enabled (default since RaspberryMatic 3.65+):
export CCU_URL='http://ccu.example.local'   # your CCU's hostname or IP
export CCU_USER='Admin'                     # default RaspberryMatic admin user
export CCU_PASSWORD='<your-ccu-password>'   # NEVER pass via --password argv
./target/release/home-inventory sync ccu
# URL/user/password come from env. CLI args also work but leak the
# password to /proc/<pid>/cmdline, ps -ef and ~/.bash_history — prefer
# env (same R-CRIT-1 reasoning as HA_TOKEN).
# If the CCU has no auth, just leave CCU_USER and CCU_PASSWORD unset.

# Hue: only if you configured a bridge file in Phase 2.
# If you omit --config, the command prints an info line and exits 0.
./target/release/home-inventory sync hue --config ~/haAutomation/home-inventory/local/hue.yml

# Shelly: mDNS discovery for 30 seconds
./target/release/home-inventory sync shelly --discover-seconds 30
```

### Notes on `INVENTORY_PUBLISH`

The inventory backend can auto-commit and push its YAML snapshots to
a configured git remote. **Do not** enable this for the first run.
If you ever do enable it, you must pass `--confirm-publish-to
'<remote-name-or-url>'` (or set `INVENTORY_PUBLISH_CONFIRM`) — without
that confirmation the sync refuses to push and exits with a clear
message:

```
INVENTORY_PUBLISH/`--publish` ist aktiv, aber `--confirm-publish-to`
(bzw. env INVENTORY_PUBLISH_CONFIRM) ist leer.
Refusing to push device inventory to remote.
Set --confirm-publish-to '<remote>' to acknowledge.
```

This is intentional: device inventory data (hostnames, MAC addresses,
firmware versions, room labels) is at least "internal" under
BSI CON.6.A1; a copy-paste accident must not turn it into a public
git commit.

## Phase 4 — Web UI (tailnet-only, NOT public)

> **CRITICAL — R-CRIT-2**: `AUTH_BYPASS=1` disables **all**
> authentication on the web UI. Anyone who can reach the listen
> address can read the full device inventory. Use it only for
> first-run inspection, only bound to the tailnet IP, and only for
> the duration of the inspection session.

```bash
# Restrict shell history for this session (defense in depth, R-LOW-1)
export HISTCONTROL=ignorespace

# AUTH_BYPASS is the PoC switch — there is no auth, the UI is
# trivially readable for anyone who can reach the bound interface.
 export AUTH_BYPASS=1

# Bind to the Tailscale IP, NOT 0.0.0.0 (R-HIGH-4). The new default
# is 127.0.0.1:8080 (loopback only), which is safe but unreachable
# from your other devices. Replace 100.x.x.x with this host's
# tailnet IP (output of `tailscale ip -4`).
./target/release/home-inventory serve --listen 100.x.x.x:8080
```

On startup, the server prints a stderr warning whenever
`AUTH_BYPASS` is active. Treat the warning as load-bearing — if you
do not see it, you have lost the indicator that auth is off.

From a tailnet-attached device, browse to `http://100.x.x.x:8080`.

To re-enable authentication, unset `AUTH_BYPASS` and put the
inventory backend behind the `authgate` sidecar (HTTP Basic with
PBKDF2-HMAC-SHA256 + signed cookie, fail-closed). See
[architecture.md](architecture.md) and the `authgate` binary under
`home-inventory/src/bin/authgate.rs`.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `bind 127.0.0.1:8080` succeeds but browser cannot reach it | Default `--listen` is loopback-only. | Re-run with `--listen 100.x.x.x:8080` (your tailnet IP). |
| `HA sync ok: 0 entities` | Token or URL wrong; or HA proxy stripped the `Authorization` header. | Verify `curl -H "Authorization: Bearer $HA_TOKEN" "$HA_URL/api/" `; check HA reverse proxy config. |
| `hue: no config provided, skipping` | `--config` not passed and `HUE_CONFIG` not set. | Either provide `~/haAutomation/home-inventory/local/hue.yml` or accept the skip (Hue is optional). |
| `hue config does not exist: <path>` | Path typo or wrong working directory. | Use an absolute path or `cd ~/haAutomation` first. |
| `Refusing to push device inventory to remote` | `INVENTORY_PUBLISH=true` without `--confirm-publish-to`. | Either `unset INVENTORY_PUBLISH` (recommended for first run) or add `--confirm-publish-to '<remote>'`. |
| `WARNUNG: AUTH_BYPASS aktiv` on every start | Expected: it is the load-bearing warning. | Either accept the warning (PoC mode) or set up `authgate` and `unset AUTH_BYPASS`. |
| Hue pairing requires bridge button press | Standard Hue UX; pairing call only works for ~30 s after the button press. | Press the bridge link button, then immediately run the `POST /api` pairing call. |
| CCU XML-API returns 404 | XML-API addon not installed on the CCU. | Install RaspberryMatic XML-API addon; verify with `curl http://<ccu>/addons/xmlapi/devicelist.cgi`. |
| `sync ccu` errors with `CCU XML-API returned <not_authenticated/>` | RaspberryMatic auth is enabled but the sync ran without credentials (or with wrong ones). | Set `CCU_USER` + `CCU_PASSWORD` env vars (do **not** pass via `--password` CLI argv — it leaks to bash history / `ps -ef`). RaspberryMatic default admin user is `Admin`. |
| `sync ccu` errors with `CCU auth misconfigured: CCU_USER is set but CCU_PASSWORD is empty` (or vice versa) | Half-configured Basic Auth. | Set both `CCU_USER` and `CCU_PASSWORD` together, or unset both for CCUs without auth. |

## What this PoC does NOT do

- No production deployment (VPS/Caddy/forward_auth path is documented
  separately in [vps-setup.md](vps-setup.md) and currently not
  recommended — see README alpha disclaimer).
- No monitoring or alerting on failed syncs.
- No backup of `inventory.db` or the YAML snapshots.
- No rotation of HA / Hue tokens (HA Long-Lived Tokens have
  effectively no expiry; rotation is operator responsibility).
- No public exposure: `--listen 127.0.0.1` (default) or
  `--listen 100.x.x.x:8080` (tailnet) keeps the UI off the public
  internet. Combined with `AUTH_BYPASS=1`, **do not** bind to
  `0.0.0.0` and **do not** publish via reverse proxy.

## After the PoC

If the run succeeded and you want to keep the setup:

- [architecture.md](architecture.md) — full architecture incl.
  authgate and trust boundaries
- [vps-setup.md](vps-setup.md) — VPS bootstrap (not currently
  recommended for production, see README disclaimer)
- [runbooks/](runbooks/) — operational procedures, e.g.
  Zigbee-coordinator replacement

## References (security mitigations)

- **R-CRIT-1** — HA token via env, not argv: OWASP ASVS V2.10.4;
  CWE-214; NIST SP 800-63B § 5.1.1.2.
- **R-CRIT-2** — `AUTH_BYPASS=1` warning + tailnet bind: OWASP ASVS
  V14.1.1, V2.1.1; NIST SP 800-53 AC-3.
- **R-HIGH-1** — `umask 077` + `chmod 600` on `hue.yml`:
  BSI ORP.4.A10; OWASP ASVS V8.1.4.
- **R-HIGH-2** — SSH key policy on the build host: BSI SYS.1.1.A15;
  CIS Linux Benchmark §5.2.
- **R-HIGH-3** — `--confirm-publish-to` for `INVENTORY_PUBLISH`:
  DSGVO Art. 5(1)(f); BSI CON.6.A1.
- **R-HIGH-4** — explicit `--listen <tailnet-ip>:port`, default
  loopback: NIST SP 800-207 § 3; BSI NET.1.1.A4.
