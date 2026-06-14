# VPS Host Setup

Operational documentation for the public VPS that runs the inventory
backend. Contains a full bootstrap guide for setting up a fresh host.

## Server context

| Property | Value |
|---|---|
| Public hostname | `<vps-host>` (e.g. `vps.example.org`) |
| Role in repo | managed by a separate Ansible repo |
| Operating system | Debian 12 (kernel 6.1.x) |
| Docker engine | 29.x |
| VPN backbone to home net | Tailscale (initial), overlays for NetBird / WireGuard exist |
| Host key Ed25519 | `<server-host-key-fingerprint>` |

## User and permission model

Deliberate separation into two Linux accounts:

| Account | Purpose | sudo? | Member of `docker`? | Login with |
|---|---|---|---|---|
| `root` | bootstrap, system packages, new users, routing config | n/a | yes | `~/.ssh/id_rsa` (workstation) |
| `deploy` | day-to-day ops: `git pull`, `docker build`, `docker compose up/down` | no | **yes** (994) | `~/.ssh/id_ed25519_vps` (workstation) |

**Why separated?** With `deploy` the entire container lifecycle runs
without root privileges. System changes require an explicit switch to
`root` and are therefore visible. The deploy keys have no write access
to `/etc` or the Docker configuration.

## Key inventory

The project creates several SSH keys, each with a clear scope. This
table is the source of truth for "which key does what":

| Private key | Where | Scope | Allows |
|---|---|---|---|
| `~/.ssh/id_rsa` (workstation) | Win | workstation identity | login `root@vps`; deploy key for the Ansible repo |
| `~/.ssh/id_ed25519_vps` (workstation) | Win | VPS login identity | login `deploy@vps` |
| `~/.ssh/id_ed25519_repo` (workstation) | Win | repo Git identity | **write** to `<github-owner>/<repo>` |
| `~/.ssh/id_ed25519_github_repo` (VPS) | Lin | VPS Git identity | **read** from `<github-owner>/<repo>` |

Each key has exactly one job. Rotations affect only one server / one
repo at a time.

## SSH setup: workstation → VPS

### Local: key + config

```bash
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_vps -N "" -C "deploy@vps"
```

Append to `~/.ssh/config`:

```
Host vps
    HostName <vps-host>
    User deploy
    IdentityFile ~/.ssh/id_ed25519_vps
    IdentitiesOnly yes
```

### Verify host key

```bash
ssh-keyscan -t ed25519 <vps-host>
# fingerprint via:
ssh-keygen -lf <(ssh-keyscan -t ed25519 <vps-host> 2>/dev/null)
```

Compare against your recorded fingerprint, then append to
`~/.ssh/known_hosts`.

### Create the deploy user on the VPS (as root)

```bash
useradd -m -s /bin/bash -G docker deploy
install -d -m700 -o deploy -g deploy /home/deploy/.ssh
echo "<contents of id_ed25519_vps.pub>" > /home/deploy/.ssh/authorized_keys
chmod 600 /home/deploy/.ssh/authorized_keys
chown deploy:deploy /home/deploy/.ssh/authorized_keys
```

Check:

```bash
ssh vps 'whoami && groups && docker ps'
```

Expected: `deploy`, groups include `docker`, `docker ps` lists existing
containers.

## SSH setup: VPS → GitHub

The VPS pulls **read-only** from the repo. The writing sync path
(`git_publish.rs`, S12) later gets its own write key or uses a PAT.

### On the VPS: key + ssh config + known_hosts

```bash
ssh vps bash <<'BASH'
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_github_repo -N "" \
  -C "deploy@vps-repo"
ssh-keyscan -t ed25519 github.com >> ~/.ssh/known_hosts

cat > ~/.ssh/config <<EOF
Host github.com
    IdentityFile ~/.ssh/id_ed25519_github_repo
    IdentitiesOnly yes
EOF
chmod 600 ~/.ssh/config
BASH
```

### Register the pubkey as a deploy key

From the workstation (gh CLI authenticated as the repo owner):

```bash
PUB=$(ssh vps 'cat ~/.ssh/id_ed25519_github_repo.pub')
gh api repos/<github-owner>/<repo>/keys \
  -f title="vps-deploy" \
  -f key="$PUB" \
  -F read_only=true
```

### Check

```bash
ssh vps 'ssh -T git@github.com'
```

Expected: `Hi <github-owner>/<repo>! You've successfully authenticated...`

## Clone the repo

```bash
ssh vps 'git clone git@github.com:<github-owner>/<repo>.git ~/<repo>'
```

Later updates:

```bash
ssh vps 'cd ~/<repo> && git pull'
```

## First image build and test

```bash
ssh vps 'cd ~/<repo>/home-inventory && \
  docker build -f docker/Dockerfile -t inventory:dev . | tail -5'
```

Check:

```bash
ssh vps 'docker image inspect inventory:dev --format "{{.Size}}"' \
  | awk '{printf "%.2f MB\n", $1/1048576}'
```

Expected: < 30 MB (NFR-9). Currently ~10 MB.

End-to-end test:

```bash
ssh vps 'docker rm -f inv-test 2>/dev/null
  docker run -d --name inv-test -p 127.0.0.1:18080:8080 inventory:dev
  sleep 2
  curl -s http://127.0.0.1:18080/health
  echo
  curl -s http://127.0.0.1:18080/api/devices
  docker rm -f inv-test'
```

Expected: `ok` and `[]`.

## Operational routines

### Full update iteration

```bash
ssh vps bash -c '
  cd ~/<repo> &&
  git pull &&
  cd home-inventory &&
  docker build -f docker/Dockerfile -t inventory:dev . &&
  docker compose -f docker/docker-compose.yml \
                 -f docker/docker-compose.vpn.tailscale.yml \
                 up -d
'
```

(Compose invocation is simplified via `just` in S13.)

### Watch logs

```bash
ssh vps 'docker logs -f inventory'
```

### Image cleanup

```bash
ssh vps 'docker image prune -f'
```

### Build-cache cleanup

```bash
ssh vps 'docker builder prune -f'
```

## Trust boundaries

```
+--------- Workstation (Windows) ---------+
|                                          |
|  id_rsa  -----------+ -> root@vps        |
|  id_ed25519_vps     + -> deploy@vps      |
|  id_ed25519_repo      -> github (write)  |
|                                          |
+--------------------|---------------------+
                     | SSH (ed25519 hostkey)
                     v
+--------- Public VPS (Debian) -----------+
|                                          |
|  root          (system, sudo)            |
|  deploy        (container lifecycle)     |
|  id_ed25519_github_repo                  |
|                  -> github (read-only)   |
|                                          |
|  ~deploy/<repo>/  <-- git pull           |
|  /etc/inventory/age.key   (planned)      |
|                                          |
+-------- Docker --------------------------+
|                                          |
|  inventory:dev   (non-root, alpine)      |
|  vpn-tailscale   (sidecar, NET_ADMIN)    |
|  caddy           (reverse proxy + TLS)   |
|                                          |
+------------------------------------------+
```

Never commit: `id_rsa`, `id_ed25519_vps`, `age.key`, and any private
keys living on the VPS.

## Planned extensions

These items are not yet present on the VPS and arrive with the
matching roadmap steps:

- `/etc/inventory/age.key` (root:deploy 0440) — for sops decryption (S8)
- Tailscale sidecar incl. auth key (S13a)
- Caddy + TLS cert + Authentik forward-auth (S14)
- DNS A record for the subdomain (S14)
- systemd service unit that runs `docker compose up -d` at boot (post-V1)
- Backup script for SQLite + compose state (post-V1)

## Disaster recovery: new VPS host

Assuming the provider gives you a fresh Debian VPS with working
networking and console / SSH access as `root`, bootstrap order:

1. **System updates and Docker install**

   ```bash
   apt update && apt -y full-upgrade
   apt -y install ca-certificates curl gnupg
   install -m 0755 -d /etc/apt/keyrings
   curl -fsSL https://download.docker.com/linux/debian/gpg \
       | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
   echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/docker.gpg] \
         https://download.docker.com/linux/debian bookworm stable" \
       > /etc/apt/sources.list.d/docker.list
   apt update
   apt -y install docker-ce docker-ce-cli containerd.io docker-compose-plugin
   ```

2. **Install workstation pubkey for root** (via provider console or password auth):

   ```bash
   mkdir -p /root/.ssh && chmod 700 /root/.ssh
   echo "<id_rsa.pub contents>" >> /root/.ssh/authorized_keys
   chmod 600 /root/.ssh/authorized_keys
   ```

3. **Create the deploy user** (see "Create the deploy user on the VPS" above)

4. **VPS-side GitHub key + ssh config** (see "SSH setup: VPS → GitHub")

5. **Clone repo, build image, test** (see above)

6. **VPN, age.key, Caddy, Authentik** per the roadmap state at recovery
   time — all configurations are tracked in the repo, only the secrets
   need to be restored from the off-repo backup (especially
   `/etc/inventory/age.key`).

## Appendix: key rotation

### Rotate the workstation-to-VPS key

```bash
# back up old key, generate new
mv ~/.ssh/id_ed25519_vps{,.old}
mv ~/.ssh/id_ed25519_vps.pub{,.old}
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_vps -N "" -C "deploy@vps"

# install new pubkey (still authenticated with the old key)
cat ~/.ssh/id_ed25519_vps.pub | ssh vps \
  'tee -a ~/.ssh/authorized_keys >/dev/null'

# verify, then remove the old entry
ssh vps 'sed -i "/$(cat ~/.ssh/id_ed25519_vps.old.pub | cut -d" " -f2 | head -c40)/d" \
  ~/.ssh/authorized_keys'

# delete old files
rm ~/.ssh/id_ed25519_vps.old*
```

### Rotate the VPS-to-GitHub key

```bash
# new key on the VPS
ssh vps 'ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_github_repo.new \
  -N "" -C "deploy@vps-repo"'

# register the new pubkey as a deploy key
PUB=$(ssh vps 'cat ~/.ssh/id_ed25519_github_repo.new.pub')
gh api repos/<github-owner>/<repo>/keys \
  -f title="vps-deploy-$(date +%Y%m%d)" -f key="$PUB" -F read_only=true

# swap in
ssh vps '
  mv ~/.ssh/id_ed25519_github_repo{,.old}
  mv ~/.ssh/id_ed25519_github_repo.new ~/.ssh/id_ed25519_github_repo
  mv ~/.ssh/id_ed25519_github_repo.new.pub ~/.ssh/id_ed25519_github_repo.pub
  ssh -T git@github.com  # smoke test
'

# delete the old deploy key on the repo (gh api)
gh api repos/<github-owner>/<repo>/keys --jq '.[] | select(.title=="vps-deploy") | .id' \
  | xargs -I{} gh api -X DELETE repos/<github-owner>/<repo>/keys/{}
```
