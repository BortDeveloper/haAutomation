# Third-Party Open Source Notices

This file lists every open source dependency of the `inventory`
workspace, with version and source. It is **automatically generated**
and **drift-checked by CI** on every push.

- **Source of truth**: `inventory/Cargo.lock` (workspace + transitive)
- **Generator**: `inventory/scripts/generate-notices.sh` (parses
  `Cargo.lock` directly; no network, no cargo required)
- **Drift check**: `.github/workflows/security.yml`, job
  `license-notices` — re-runs the generator and `diff`s against
  this committed file. CI fails on any divergence.
- **License enforcement**: `inventory/deny.toml` (`cargo deny check
  licenses`) restricts the accepted SPDX set to permissive licenses
  (MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-DFS-2016, CC0-1.0,
  Zlib). Copyleft is denied. License compliance is therefore enforced
  by `cargo deny` even though the per-crate SPDX string is not
  inlined into this file (Cargo.lock does not record it).

If you add, remove, or update a dependency:

```
bash inventory/scripts/generate-notices.sh > THIRD-PARTY-NOTICES.md
```

…and commit the regenerated file alongside your `Cargo.lock` change.
CI will block the merge if you forget.

## Summary

- Total entries in `Cargo.lock`: 158
- Workspace crates (this repo): 1
- **Third-party dependencies**: 157

## Accepted licenses

The following SPDX identifiers are accepted by the project (see
`inventory/deny.toml` and `inventory/about.toml`):

- MIT
- Apache-2.0 (also `Apache-2.0 WITH LLVM-exception`)
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Unicode-DFS-2016
- CC0-1.0
- Zlib

Any dependency under a non-listed license will fail
`cargo deny check licenses` in CI before it can be merged.

## Dependencies

| # | Crate | Version | Source | crates.io |
|---|-------|---------|--------|-----------|
| 1 | ahash | 0.8.12 | crates.io | https://crates.io/crates/ahash/0.8.12 |
| 2 | anstream | 1.0.0 | crates.io | https://crates.io/crates/anstream/1.0.0 |
| 3 | anstyle | 1.0.14 | crates.io | https://crates.io/crates/anstyle/1.0.14 |
| 4 | anstyle-parse | 1.0.0 | crates.io | https://crates.io/crates/anstyle-parse/1.0.0 |
| 5 | anstyle-query | 1.1.5 | crates.io | https://crates.io/crates/anstyle-query/1.1.5 |
| 6 | anstyle-wincon | 3.0.11 | crates.io | https://crates.io/crates/anstyle-wincon/3.0.11 |
| 7 | anyhow | 1.0.102 | crates.io | https://crates.io/crates/anyhow/1.0.102 |
| 8 | ascii | 1.1.0 | crates.io | https://crates.io/crates/ascii/1.1.0 |
| 9 | base64 | 0.22.1 | crates.io | https://crates.io/crates/base64/0.22.1 |
| 10 | bitflags | 2.11.1 | crates.io | https://crates.io/crates/bitflags/2.11.1 |
| 11 | block-buffer | 0.10.4 | crates.io | https://crates.io/crates/block-buffer/0.10.4 |
| 12 | cc | 1.2.62 | crates.io | https://crates.io/crates/cc/1.2.62 |
| 13 | cfg-if | 1.0.4 | crates.io | https://crates.io/crates/cfg-if/1.0.4 |
| 14 | chunked_transfer | 1.5.0 | crates.io | https://crates.io/crates/chunked_transfer/1.5.0 |
| 15 | clap | 4.6.1 | crates.io | https://crates.io/crates/clap/4.6.1 |
| 16 | clap_builder | 4.6.0 | crates.io | https://crates.io/crates/clap_builder/4.6.0 |
| 17 | clap_derive | 4.6.1 | crates.io | https://crates.io/crates/clap_derive/4.6.1 |
| 18 | clap_lex | 1.1.0 | crates.io | https://crates.io/crates/clap_lex/1.1.0 |
| 19 | colorchoice | 1.0.5 | crates.io | https://crates.io/crates/colorchoice/1.0.5 |
| 20 | cpufeatures | 0.2.17 | crates.io | https://crates.io/crates/cpufeatures/0.2.17 |
| 21 | crypto-common | 0.1.7 | crates.io | https://crates.io/crates/crypto-common/0.1.7 |
| 22 | digest | 0.10.7 | crates.io | https://crates.io/crates/digest/0.10.7 |
| 23 | displaydoc | 0.2.5 | crates.io | https://crates.io/crates/displaydoc/0.2.5 |
| 24 | equivalent | 1.0.2 | crates.io | https://crates.io/crates/equivalent/1.0.2 |
| 25 | errno | 0.3.14 | crates.io | https://crates.io/crates/errno/0.3.14 |
| 26 | fallible-iterator | 0.3.0 | crates.io | https://crates.io/crates/fallible-iterator/0.3.0 |
| 27 | fallible-streaming-iterator | 0.1.9 | crates.io | https://crates.io/crates/fallible-streaming-iterator/0.1.9 |
| 28 | fastrand | 2.4.1 | crates.io | https://crates.io/crates/fastrand/2.4.1 |
| 29 | find-msvc-tools | 0.1.9 | crates.io | https://crates.io/crates/find-msvc-tools/0.1.9 |
| 30 | flume | 0.11.1 | crates.io | https://crates.io/crates/flume/0.11.1 |
| 31 | foldhash | 0.1.5 | crates.io | https://crates.io/crates/foldhash/0.1.5 |
| 32 | form_urlencoded | 1.2.2 | crates.io | https://crates.io/crates/form_urlencoded/1.2.2 |
| 33 | futures-core | 0.3.32 | crates.io | https://crates.io/crates/futures-core/0.3.32 |
| 34 | futures-sink | 0.3.32 | crates.io | https://crates.io/crates/futures-sink/0.3.32 |
| 35 | generic-array | 0.14.7 | crates.io | https://crates.io/crates/generic-array/0.14.7 |
| 36 | getrandom | 0.2.17 | crates.io | https://crates.io/crates/getrandom/0.2.17 |
| 37 | getrandom | 0.4.2 | crates.io | https://crates.io/crates/getrandom/0.4.2 |
| 38 | hashbrown | 0.14.5 | crates.io | https://crates.io/crates/hashbrown/0.14.5 |
| 39 | hashbrown | 0.15.5 | crates.io | https://crates.io/crates/hashbrown/0.15.5 |
| 40 | hashbrown | 0.17.1 | crates.io | https://crates.io/crates/hashbrown/0.17.1 |
| 41 | hashlink | 0.9.1 | crates.io | https://crates.io/crates/hashlink/0.9.1 |
| 42 | heck | 0.5.0 | crates.io | https://crates.io/crates/heck/0.5.0 |
| 43 | hmac | 0.12.1 | crates.io | https://crates.io/crates/hmac/0.12.1 |
| 44 | httpdate | 1.0.3 | crates.io | https://crates.io/crates/httpdate/1.0.3 |
| 45 | icu_collections | 2.2.0 | crates.io | https://crates.io/crates/icu_collections/2.2.0 |
| 46 | icu_locale_core | 2.2.0 | crates.io | https://crates.io/crates/icu_locale_core/2.2.0 |
| 47 | icu_normalizer | 2.2.0 | crates.io | https://crates.io/crates/icu_normalizer/2.2.0 |
| 48 | icu_normalizer_data | 2.2.0 | crates.io | https://crates.io/crates/icu_normalizer_data/2.2.0 |
| 49 | icu_properties | 2.2.0 | crates.io | https://crates.io/crates/icu_properties/2.2.0 |
| 50 | icu_properties_data | 2.2.0 | crates.io | https://crates.io/crates/icu_properties_data/2.2.0 |
| 51 | icu_provider | 2.2.0 | crates.io | https://crates.io/crates/icu_provider/2.2.0 |
| 52 | id-arena | 2.3.0 | crates.io | https://crates.io/crates/id-arena/2.3.0 |
| 53 | idna | 1.1.0 | crates.io | https://crates.io/crates/idna/1.1.0 |
| 54 | idna_adapter | 1.2.2 | crates.io | https://crates.io/crates/idna_adapter/1.2.2 |
| 55 | if-addrs | 0.13.4 | crates.io | https://crates.io/crates/if-addrs/0.13.4 |
| 56 | indexmap | 2.14.0 | crates.io | https://crates.io/crates/indexmap/2.14.0 |
| 57 | inventory | 0.1.0 | _workspace (this repo)_ | n/a |
| 58 | is_terminal_polyfill | 1.70.2 | crates.io | https://crates.io/crates/is_terminal_polyfill/1.70.2 |
| 59 | itoa | 1.0.18 | crates.io | https://crates.io/crates/itoa/1.0.18 |
| 60 | leb128fmt | 0.1.0 | crates.io | https://crates.io/crates/leb128fmt/0.1.0 |
| 61 | libc | 0.2.186 | crates.io | https://crates.io/crates/libc/0.2.186 |
| 62 | libsqlite3-sys | 0.28.0 | crates.io | https://crates.io/crates/libsqlite3-sys/0.28.0 |
| 63 | linux-raw-sys | 0.12.1 | crates.io | https://crates.io/crates/linux-raw-sys/0.12.1 |
| 64 | litemap | 0.8.2 | crates.io | https://crates.io/crates/litemap/0.8.2 |
| 65 | lock_api | 0.4.14 | crates.io | https://crates.io/crates/lock_api/0.4.14 |
| 66 | log | 0.4.29 | crates.io | https://crates.io/crates/log/0.4.29 |
| 67 | mdns-sd | 0.13.11 | crates.io | https://crates.io/crates/mdns-sd/0.13.11 |
| 68 | memchr | 2.8.0 | crates.io | https://crates.io/crates/memchr/2.8.0 |
| 69 | mio | 1.2.0 | crates.io | https://crates.io/crates/mio/1.2.0 |
| 70 | once_cell | 1.21.4 | crates.io | https://crates.io/crates/once_cell/1.21.4 |
| 71 | once_cell_polyfill | 1.70.2 | crates.io | https://crates.io/crates/once_cell_polyfill/1.70.2 |
| 72 | pbkdf2 | 0.12.2 | crates.io | https://crates.io/crates/pbkdf2/0.12.2 |
| 73 | percent-encoding | 2.3.2 | crates.io | https://crates.io/crates/percent-encoding/2.3.2 |
| 74 | pkg-config | 0.3.33 | crates.io | https://crates.io/crates/pkg-config/0.3.33 |
| 75 | potential_utf | 0.1.5 | crates.io | https://crates.io/crates/potential_utf/0.1.5 |
| 76 | prettyplease | 0.2.37 | crates.io | https://crates.io/crates/prettyplease/0.2.37 |
| 77 | proc-macro2 | 1.0.106 | crates.io | https://crates.io/crates/proc-macro2/1.0.106 |
| 78 | quote | 1.0.45 | crates.io | https://crates.io/crates/quote/1.0.45 |
| 79 | r-efi | 6.0.0 | crates.io | https://crates.io/crates/r-efi/6.0.0 |
| 80 | ring | 0.17.14 | crates.io | https://crates.io/crates/ring/0.17.14 |
| 81 | roxmltree | 0.20.0 | crates.io | https://crates.io/crates/roxmltree/0.20.0 |
| 82 | rusqlite | 0.31.0 | crates.io | https://crates.io/crates/rusqlite/0.31.0 |
| 83 | rustix | 1.1.4 | crates.io | https://crates.io/crates/rustix/1.1.4 |
| 84 | rustls | 0.23.40 | crates.io | https://crates.io/crates/rustls/0.23.40 |
| 85 | rustls-pki-types | 1.14.1 | crates.io | https://crates.io/crates/rustls-pki-types/1.14.1 |
| 86 | rustls-webpki | 0.103.13 | crates.io | https://crates.io/crates/rustls-webpki/0.103.13 |
| 87 | ryu | 1.0.23 | crates.io | https://crates.io/crates/ryu/1.0.23 |
| 88 | scopeguard | 1.2.0 | crates.io | https://crates.io/crates/scopeguard/1.2.0 |
| 89 | semver | 1.0.28 | crates.io | https://crates.io/crates/semver/1.0.28 |
| 90 | serde | 1.0.228 | crates.io | https://crates.io/crates/serde/1.0.228 |
| 91 | serde_core | 1.0.228 | crates.io | https://crates.io/crates/serde_core/1.0.228 |
| 92 | serde_derive | 1.0.228 | crates.io | https://crates.io/crates/serde_derive/1.0.228 |
| 93 | serde_json | 1.0.149 | crates.io | https://crates.io/crates/serde_json/1.0.149 |
| 94 | serde_yaml_ng | 0.10.0 | crates.io | https://crates.io/crates/serde_yaml_ng/0.10.0 |
| 95 | sha2 | 0.10.9 | crates.io | https://crates.io/crates/sha2/0.10.9 |
| 96 | shlex | 1.3.0 | crates.io | https://crates.io/crates/shlex/1.3.0 |
| 97 | smallvec | 1.15.1 | crates.io | https://crates.io/crates/smallvec/1.15.1 |
| 98 | socket2 | 0.5.10 | crates.io | https://crates.io/crates/socket2/0.5.10 |
| 99 | spin | 0.9.8 | crates.io | https://crates.io/crates/spin/0.9.8 |
| 100 | stable_deref_trait | 1.2.1 | crates.io | https://crates.io/crates/stable_deref_trait/1.2.1 |
| 101 | strsim | 0.11.1 | crates.io | https://crates.io/crates/strsim/0.11.1 |
| 102 | subtle | 2.6.1 | crates.io | https://crates.io/crates/subtle/2.6.1 |
| 103 | syn | 2.0.117 | crates.io | https://crates.io/crates/syn/2.0.117 |
| 104 | synstructure | 0.13.2 | crates.io | https://crates.io/crates/synstructure/0.13.2 |
| 105 | tempfile | 3.27.0 | crates.io | https://crates.io/crates/tempfile/3.27.0 |
| 106 | tiny_http | 0.12.0 | crates.io | https://crates.io/crates/tiny_http/0.12.0 |
| 107 | tinystr | 0.8.3 | crates.io | https://crates.io/crates/tinystr/0.8.3 |
| 108 | typenum | 1.20.0 | crates.io | https://crates.io/crates/typenum/1.20.0 |
| 109 | unicode-ident | 1.0.24 | crates.io | https://crates.io/crates/unicode-ident/1.0.24 |
| 110 | unicode-xid | 0.2.6 | crates.io | https://crates.io/crates/unicode-xid/0.2.6 |
| 111 | unsafe-libyaml | 0.2.11 | crates.io | https://crates.io/crates/unsafe-libyaml/0.2.11 |
| 112 | untrusted | 0.9.0 | crates.io | https://crates.io/crates/untrusted/0.9.0 |
| 113 | ureq | 2.12.1 | crates.io | https://crates.io/crates/ureq/2.12.1 |
| 114 | url | 2.5.8 | crates.io | https://crates.io/crates/url/2.5.8 |
| 115 | utf8_iter | 1.0.4 | crates.io | https://crates.io/crates/utf8_iter/1.0.4 |
| 116 | utf8parse | 0.2.2 | crates.io | https://crates.io/crates/utf8parse/0.2.2 |
| 117 | vcpkg | 0.2.15 | crates.io | https://crates.io/crates/vcpkg/0.2.15 |
| 118 | version_check | 0.9.5 | crates.io | https://crates.io/crates/version_check/0.9.5 |
| 119 | wasi | 0.11.1+wasi-snapshot-preview1 | crates.io | https://crates.io/crates/wasi/0.11.1+wasi-snapshot-preview1 |
| 120 | wasip2 | 1.0.3+wasi-0.2.9 | crates.io | https://crates.io/crates/wasip2/1.0.3+wasi-0.2.9 |
| 121 | wasip3 | 0.4.0+wasi-0.3.0-rc-2026-01-06 | crates.io | https://crates.io/crates/wasip3/0.4.0+wasi-0.3.0-rc-2026-01-06 |
| 122 | wasm-encoder | 0.244.0 | crates.io | https://crates.io/crates/wasm-encoder/0.244.0 |
| 123 | wasm-metadata | 0.244.0 | crates.io | https://crates.io/crates/wasm-metadata/0.244.0 |
| 124 | wasmparser | 0.244.0 | crates.io | https://crates.io/crates/wasmparser/0.244.0 |
| 125 | webpki-roots | 0.26.11 | crates.io | https://crates.io/crates/webpki-roots/0.26.11 |
| 126 | webpki-roots | 1.0.7 | crates.io | https://crates.io/crates/webpki-roots/1.0.7 |
| 127 | windows-link | 0.2.1 | crates.io | https://crates.io/crates/windows-link/0.2.1 |
| 128 | windows-sys | 0.52.0 | crates.io | https://crates.io/crates/windows-sys/0.52.0 |
| 129 | windows-sys | 0.59.0 | crates.io | https://crates.io/crates/windows-sys/0.59.0 |
| 130 | windows-sys | 0.61.2 | crates.io | https://crates.io/crates/windows-sys/0.61.2 |
| 131 | windows-targets | 0.52.6 | crates.io | https://crates.io/crates/windows-targets/0.52.6 |
| 132 | windows_aarch64_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_aarch64_gnullvm/0.52.6 |
| 133 | windows_aarch64_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_aarch64_msvc/0.52.6 |
| 134 | windows_i686_gnu | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_gnu/0.52.6 |
| 135 | windows_i686_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_gnullvm/0.52.6 |
| 136 | windows_i686_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_msvc/0.52.6 |
| 137 | windows_x86_64_gnu | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_gnu/0.52.6 |
| 138 | windows_x86_64_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_gnullvm/0.52.6 |
| 139 | windows_x86_64_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_msvc/0.52.6 |
| 140 | wit-bindgen | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen/0.51.0 |
| 141 | wit-bindgen | 0.57.1 | crates.io | https://crates.io/crates/wit-bindgen/0.57.1 |
| 142 | wit-bindgen-core | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-core/0.51.0 |
| 143 | wit-bindgen-rust | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-rust/0.51.0 |
| 144 | wit-bindgen-rust-macro | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-rust-macro/0.51.0 |
| 145 | wit-component | 0.244.0 | crates.io | https://crates.io/crates/wit-component/0.244.0 |
| 146 | wit-parser | 0.244.0 | crates.io | https://crates.io/crates/wit-parser/0.244.0 |
| 147 | writeable | 0.6.3 | crates.io | https://crates.io/crates/writeable/0.6.3 |
| 148 | yoke | 0.8.2 | crates.io | https://crates.io/crates/yoke/0.8.2 |
| 149 | yoke-derive | 0.8.2 | crates.io | https://crates.io/crates/yoke-derive/0.8.2 |
| 150 | zerocopy | 0.8.48 | crates.io | https://crates.io/crates/zerocopy/0.8.48 |
| 151 | zerocopy-derive | 0.8.48 | crates.io | https://crates.io/crates/zerocopy-derive/0.8.48 |
| 152 | zerofrom | 0.1.8 | crates.io | https://crates.io/crates/zerofrom/0.1.8 |
| 153 | zerofrom-derive | 0.1.7 | crates.io | https://crates.io/crates/zerofrom-derive/0.1.7 |
| 154 | zeroize | 1.8.2 | crates.io | https://crates.io/crates/zeroize/1.8.2 |
| 155 | zerotrie | 0.2.4 | crates.io | https://crates.io/crates/zerotrie/0.2.4 |
| 156 | zerovec | 0.11.6 | crates.io | https://crates.io/crates/zerovec/0.11.6 |
| 157 | zerovec-derive | 0.11.3 | crates.io | https://crates.io/crates/zerovec-derive/0.11.3 |
| 158 | zmij | 1.0.21 | crates.io | https://crates.io/crates/zmij/1.0.21 |

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
