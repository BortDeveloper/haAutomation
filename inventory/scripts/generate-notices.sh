#!/usr/bin/env bash
# Generate THIRD-PARTY-NOTICES.md from inventory/Cargo.lock.
#
# Why a shell script and not `cargo about generate`?
#   - Deterministic: parses Cargo.lock directly, no network lookups
#     against crates.io, no cache state, no toolchain dependency.
#   - Reproducible across environments (CI, dev laptop, edge host)
#     with only POSIX coreutils + awk.
#   - The CI drift check (.github/workflows/security.yml job
#     `license-notices`) re-runs this script and `diff`s the result
#     against the committed file. Any mismatch fails the build.
#
# License data limitation:
#   Cargo.lock does NOT record SPDX license strings. The authoritative
#   per-crate license is the `license` field in each crate's
#   Cargo.toml on crates.io. We list the crates with a direct link;
#   the cargo-deny `[licenses]` gate (deny.toml) enforces that only
#   permissive licenses are accepted at build time, so license
#   compliance is enforced separately even though it is not embedded
#   inline here.
#
# Usage (from the repository root):
#   bash inventory/scripts/generate-notices.sh > THIRD-PARTY-NOTICES.md
#
# Usage (from inventory/):
#   bash scripts/generate-notices.sh > ../THIRD-PARTY-NOTICES.md

set -euo pipefail

# Resolve repository root regardless of where the script is invoked
# from (CI runs with working-directory: inventory; local devs may run
# from the repo root).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INVENTORY_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_ROOT="$(cd "${INVENTORY_DIR}/.." && pwd)"
LOCK_FILE="${INVENTORY_DIR}/Cargo.lock"

if [[ ! -f "${LOCK_FILE}" ]]; then
    echo "error: ${LOCK_FILE} not found" >&2
    exit 2
fi

# Count distinct (name, version) pairs from Cargo.lock for the header.
CRATE_COUNT="$(awk '/^\[\[package\]\]/{c++} END{print c+0}' "${LOCK_FILE}")"
# Drop the workspace crate itself ("inventory") from the public count
# of third-party dependencies.
WORKSPACE_CRATES=1
THIRD_PARTY_COUNT=$((CRATE_COUNT - WORKSPACE_CRATES))

cat <<EOF
# Third-Party Open Source Notices

This file lists every open source dependency of the \`inventory\`
workspace, with version and source. It is **automatically generated**
and **drift-checked by CI** on every push.

- **Source of truth**: \`inventory/Cargo.lock\` (workspace + transitive)
- **Generator**: \`inventory/scripts/generate-notices.sh\` (parses
  \`Cargo.lock\` directly; no network, no cargo required)
- **Drift check**: \`.github/workflows/security.yml\`, job
  \`license-notices\` — re-runs the generator and \`diff\`s against
  this committed file. CI fails on any divergence.
- **License enforcement**: \`inventory/deny.toml\` (\`cargo deny check
  licenses\`) restricts the accepted SPDX set to permissive licenses
  (MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-DFS-2016, CC0-1.0,
  Zlib). Copyleft is denied. License compliance is therefore enforced
  by \`cargo deny\` even though the per-crate SPDX string is not
  inlined into this file (Cargo.lock does not record it).

If you add, remove, or update a dependency:

\`\`\`
bash inventory/scripts/generate-notices.sh > THIRD-PARTY-NOTICES.md
\`\`\`

…and commit the regenerated file alongside your \`Cargo.lock\` change.
CI will block the merge if you forget.

## Summary

- Total entries in \`Cargo.lock\`: ${CRATE_COUNT}
- Workspace crates (this repo): ${WORKSPACE_CRATES}
- **Third-party dependencies**: ${THIRD_PARTY_COUNT}

## Accepted licenses

The following SPDX identifiers are accepted by the project (see
\`inventory/deny.toml\` and \`inventory/about.toml\`):

- MIT
- Apache-2.0 (also \`Apache-2.0 WITH LLVM-exception\`)
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Unicode-DFS-2016
- CC0-1.0
- Zlib

Any dependency under a non-listed license will fail
\`cargo deny check licenses\` in CI before it can be merged.

## Dependencies

| # | Crate | Version | Source | crates.io |
|---|-------|---------|--------|-----------|
EOF

# Parse Cargo.lock with awk. The TOML format is regular enough for
# this restricted use case: each [[package]] block has `name`,
# `version`, and (for crates.io deps) `source = "registry+..."`.
# Workspace-local crates have no `source` line; we still list them so
# the file is exhaustive, marked as "workspace".
#
# Output is sorted by crate name + version for a stable diff.
awk '
    BEGIN {
        in_pkg = 0; name = ""; version = ""; source = ""
    }
    /^\[\[package\]\]/ {
        if (in_pkg && name != "") {
            print name "\t" version "\t" source
        }
        in_pkg = 1; name = ""; version = ""; source = "workspace"
        next
    }
    /^name = / && in_pkg {
        sub(/^name = "/, ""); sub(/"$/, ""); name = $0; next
    }
    /^version = / && in_pkg {
        sub(/^version = "/, ""); sub(/"$/, ""); version = $0; next
    }
    /^source = / && in_pkg {
        sub(/^source = "/, ""); sub(/"$/, ""); source = $0; next
    }
    END {
        if (in_pkg && name != "") {
            print name "\t" version "\t" source
        }
    }
' "${LOCK_FILE}" | LC_ALL=C sort -t $'\t' -k1,1 -k2,2 | awk -F'\t' '
    BEGIN { n = 0 }
    {
        n++
        name = $1; version = $2; source = $3
        if (source == "workspace") {
            src_label = "_workspace (this repo)_"
            crates_io = "n/a"
        } else if (index(source, "registry+https://github.com/rust-lang/crates.io-index") > 0) {
            src_label = "crates.io"
            crates_io = "https://crates.io/crates/" name "/" version
        } else {
            # Any non-crates.io source would already be blocked by
            # cargo-deny [sources].unknown-registry = "deny", but we
            # surface it explicitly here for visibility.
            src_label = source
            crates_io = "n/a"
        }
        printf("| %d | %s | %s | %s | %s |\n", n, name, version, src_label, crates_io)
    }
'

cat <<'EOF'

---

For each dependency, the canonical license text is published on its
crates.io page (linked above) and bundled with the crate source in
`~/.cargo/registry/src/`. To see the SPDX license string for a
specific crate locally:

```
cargo metadata --format-version 1 --manifest-path inventory/Cargo.toml \
    | jq -r '.packages[] | "\(.name) \(.version) \(.license // .license_file // "UNKNOWN")"' \
    | sort -u
```

CI runs `cargo deny check licenses` on every push, so any crate
without an accepted license string in its `Cargo.toml` will block
the build.
EOF
