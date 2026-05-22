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

- Total entries in `Cargo.lock`: 165
- Workspace crates (this repo): 1
- **Third-party dependencies**: 164

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
| 12 | block-buffer | 0.12.0 | crates.io | https://crates.io/crates/block-buffer/0.12.0 |
| 13 | cc | 1.2.62 | crates.io | https://crates.io/crates/cc/1.2.62 |
| 14 | cfg-if | 1.0.4 | crates.io | https://crates.io/crates/cfg-if/1.0.4 |
| 15 | chunked_transfer | 1.5.0 | crates.io | https://crates.io/crates/chunked_transfer/1.5.0 |
| 16 | clap | 4.6.1 | crates.io | https://crates.io/crates/clap/4.6.1 |
| 17 | clap_builder | 4.6.0 | crates.io | https://crates.io/crates/clap_builder/4.6.0 |
| 18 | clap_derive | 4.6.1 | crates.io | https://crates.io/crates/clap_derive/4.6.1 |
| 19 | clap_lex | 1.1.0 | crates.io | https://crates.io/crates/clap_lex/1.1.0 |
| 20 | cmov | 0.5.3 | crates.io | https://crates.io/crates/cmov/0.5.3 |
| 21 | colorchoice | 1.0.5 | crates.io | https://crates.io/crates/colorchoice/1.0.5 |
| 22 | cpufeatures | 0.2.17 | crates.io | https://crates.io/crates/cpufeatures/0.2.17 |
| 23 | crypto-common | 0.1.7 | crates.io | https://crates.io/crates/crypto-common/0.1.7 |
| 24 | crypto-common | 0.2.2 | crates.io | https://crates.io/crates/crypto-common/0.2.2 |
| 25 | ctutils | 0.4.2 | crates.io | https://crates.io/crates/ctutils/0.4.2 |
| 26 | digest | 0.10.7 | crates.io | https://crates.io/crates/digest/0.10.7 |
| 27 | digest | 0.11.3 | crates.io | https://crates.io/crates/digest/0.11.3 |
| 28 | displaydoc | 0.2.5 | crates.io | https://crates.io/crates/displaydoc/0.2.5 |
| 29 | equivalent | 1.0.2 | crates.io | https://crates.io/crates/equivalent/1.0.2 |
| 30 | errno | 0.3.14 | crates.io | https://crates.io/crates/errno/0.3.14 |
| 31 | fallible-iterator | 0.3.0 | crates.io | https://crates.io/crates/fallible-iterator/0.3.0 |
| 32 | fallible-streaming-iterator | 0.1.9 | crates.io | https://crates.io/crates/fallible-streaming-iterator/0.1.9 |
| 33 | fastrand | 2.4.1 | crates.io | https://crates.io/crates/fastrand/2.4.1 |
| 34 | find-msvc-tools | 0.1.9 | crates.io | https://crates.io/crates/find-msvc-tools/0.1.9 |
| 35 | flume | 0.11.1 | crates.io | https://crates.io/crates/flume/0.11.1 |
| 36 | foldhash | 0.1.5 | crates.io | https://crates.io/crates/foldhash/0.1.5 |
| 37 | form_urlencoded | 1.2.2 | crates.io | https://crates.io/crates/form_urlencoded/1.2.2 |
| 38 | futures-core | 0.3.32 | crates.io | https://crates.io/crates/futures-core/0.3.32 |
| 39 | futures-sink | 0.3.32 | crates.io | https://crates.io/crates/futures-sink/0.3.32 |
| 40 | generic-array | 0.14.7 | crates.io | https://crates.io/crates/generic-array/0.14.7 |
| 41 | getrandom | 0.2.17 | crates.io | https://crates.io/crates/getrandom/0.2.17 |
| 42 | getrandom | 0.4.2 | crates.io | https://crates.io/crates/getrandom/0.4.2 |
| 43 | hashbrown | 0.14.5 | crates.io | https://crates.io/crates/hashbrown/0.14.5 |
| 44 | hashbrown | 0.15.5 | crates.io | https://crates.io/crates/hashbrown/0.15.5 |
| 45 | hashbrown | 0.17.1 | crates.io | https://crates.io/crates/hashbrown/0.17.1 |
| 46 | hashlink | 0.9.1 | crates.io | https://crates.io/crates/hashlink/0.9.1 |
| 47 | heck | 0.5.0 | crates.io | https://crates.io/crates/heck/0.5.0 |
| 48 | hmac | 0.12.1 | crates.io | https://crates.io/crates/hmac/0.12.1 |
| 49 | hmac | 0.13.0 | crates.io | https://crates.io/crates/hmac/0.13.0 |
| 50 | httpdate | 1.0.3 | crates.io | https://crates.io/crates/httpdate/1.0.3 |
| 51 | hybrid-array | 0.4.12 | crates.io | https://crates.io/crates/hybrid-array/0.4.12 |
| 52 | icu_collections | 2.2.0 | crates.io | https://crates.io/crates/icu_collections/2.2.0 |
| 53 | icu_locale_core | 2.2.0 | crates.io | https://crates.io/crates/icu_locale_core/2.2.0 |
| 54 | icu_normalizer | 2.2.0 | crates.io | https://crates.io/crates/icu_normalizer/2.2.0 |
| 55 | icu_normalizer_data | 2.2.0 | crates.io | https://crates.io/crates/icu_normalizer_data/2.2.0 |
| 56 | icu_properties | 2.2.0 | crates.io | https://crates.io/crates/icu_properties/2.2.0 |
| 57 | icu_properties_data | 2.2.0 | crates.io | https://crates.io/crates/icu_properties_data/2.2.0 |
| 58 | icu_provider | 2.2.0 | crates.io | https://crates.io/crates/icu_provider/2.2.0 |
| 59 | id-arena | 2.3.0 | crates.io | https://crates.io/crates/id-arena/2.3.0 |
| 60 | idna | 1.1.0 | crates.io | https://crates.io/crates/idna/1.1.0 |
| 61 | idna_adapter | 1.2.2 | crates.io | https://crates.io/crates/idna_adapter/1.2.2 |
| 62 | if-addrs | 0.13.4 | crates.io | https://crates.io/crates/if-addrs/0.13.4 |
| 63 | indexmap | 2.14.0 | crates.io | https://crates.io/crates/indexmap/2.14.0 |
| 64 | inventory | 0.1.0 | _workspace (this repo)_ | n/a |
| 65 | is_terminal_polyfill | 1.70.2 | crates.io | https://crates.io/crates/is_terminal_polyfill/1.70.2 |
| 66 | itoa | 1.0.18 | crates.io | https://crates.io/crates/itoa/1.0.18 |
| 67 | leb128fmt | 0.1.0 | crates.io | https://crates.io/crates/leb128fmt/0.1.0 |
| 68 | libc | 0.2.186 | crates.io | https://crates.io/crates/libc/0.2.186 |
| 69 | libsqlite3-sys | 0.28.0 | crates.io | https://crates.io/crates/libsqlite3-sys/0.28.0 |
| 70 | linux-raw-sys | 0.12.1 | crates.io | https://crates.io/crates/linux-raw-sys/0.12.1 |
| 71 | litemap | 0.8.2 | crates.io | https://crates.io/crates/litemap/0.8.2 |
| 72 | lock_api | 0.4.14 | crates.io | https://crates.io/crates/lock_api/0.4.14 |
| 73 | log | 0.4.29 | crates.io | https://crates.io/crates/log/0.4.29 |
| 74 | mdns-sd | 0.13.11 | crates.io | https://crates.io/crates/mdns-sd/0.13.11 |
| 75 | memchr | 2.8.0 | crates.io | https://crates.io/crates/memchr/2.8.0 |
| 76 | mio | 1.2.0 | crates.io | https://crates.io/crates/mio/1.2.0 |
| 77 | once_cell | 1.21.4 | crates.io | https://crates.io/crates/once_cell/1.21.4 |
| 78 | once_cell_polyfill | 1.70.2 | crates.io | https://crates.io/crates/once_cell_polyfill/1.70.2 |
| 79 | pbkdf2 | 0.12.2 | crates.io | https://crates.io/crates/pbkdf2/0.12.2 |
| 80 | percent-encoding | 2.3.2 | crates.io | https://crates.io/crates/percent-encoding/2.3.2 |
| 81 | pkg-config | 0.3.33 | crates.io | https://crates.io/crates/pkg-config/0.3.33 |
| 82 | potential_utf | 0.1.5 | crates.io | https://crates.io/crates/potential_utf/0.1.5 |
| 83 | prettyplease | 0.2.37 | crates.io | https://crates.io/crates/prettyplease/0.2.37 |
| 84 | proc-macro2 | 1.0.106 | crates.io | https://crates.io/crates/proc-macro2/1.0.106 |
| 85 | quote | 1.0.45 | crates.io | https://crates.io/crates/quote/1.0.45 |
| 86 | r-efi | 6.0.0 | crates.io | https://crates.io/crates/r-efi/6.0.0 |
| 87 | ring | 0.17.14 | crates.io | https://crates.io/crates/ring/0.17.14 |
| 88 | roxmltree | 0.21.1 | crates.io | https://crates.io/crates/roxmltree/0.21.1 |
| 89 | rusqlite | 0.31.0 | crates.io | https://crates.io/crates/rusqlite/0.31.0 |
| 90 | rustix | 1.1.4 | crates.io | https://crates.io/crates/rustix/1.1.4 |
| 91 | rustls | 0.23.40 | crates.io | https://crates.io/crates/rustls/0.23.40 |
| 92 | rustls-pki-types | 1.14.1 | crates.io | https://crates.io/crates/rustls-pki-types/1.14.1 |
| 93 | rustls-webpki | 0.103.13 | crates.io | https://crates.io/crates/rustls-webpki/0.103.13 |
| 94 | ryu | 1.0.23 | crates.io | https://crates.io/crates/ryu/1.0.23 |
| 95 | scopeguard | 1.2.0 | crates.io | https://crates.io/crates/scopeguard/1.2.0 |
| 96 | semver | 1.0.28 | crates.io | https://crates.io/crates/semver/1.0.28 |
| 97 | serde | 1.0.228 | crates.io | https://crates.io/crates/serde/1.0.228 |
| 98 | serde_core | 1.0.228 | crates.io | https://crates.io/crates/serde_core/1.0.228 |
| 99 | serde_derive | 1.0.228 | crates.io | https://crates.io/crates/serde_derive/1.0.228 |
| 100 | serde_json | 1.0.149 | crates.io | https://crates.io/crates/serde_json/1.0.149 |
| 101 | serde_yaml_ng | 0.10.0 | crates.io | https://crates.io/crates/serde_yaml_ng/0.10.0 |
| 102 | sha2 | 0.10.9 | crates.io | https://crates.io/crates/sha2/0.10.9 |
| 103 | shlex | 1.3.0 | crates.io | https://crates.io/crates/shlex/1.3.0 |
| 104 | smallvec | 1.15.1 | crates.io | https://crates.io/crates/smallvec/1.15.1 |
| 105 | socket2 | 0.5.10 | crates.io | https://crates.io/crates/socket2/0.5.10 |
| 106 | spin | 0.9.8 | crates.io | https://crates.io/crates/spin/0.9.8 |
| 107 | stable_deref_trait | 1.2.1 | crates.io | https://crates.io/crates/stable_deref_trait/1.2.1 |
| 108 | strsim | 0.11.1 | crates.io | https://crates.io/crates/strsim/0.11.1 |
| 109 | subtle | 2.6.1 | crates.io | https://crates.io/crates/subtle/2.6.1 |
| 110 | syn | 2.0.117 | crates.io | https://crates.io/crates/syn/2.0.117 |
| 111 | synstructure | 0.13.2 | crates.io | https://crates.io/crates/synstructure/0.13.2 |
| 112 | tempfile | 3.27.0 | crates.io | https://crates.io/crates/tempfile/3.27.0 |
| 113 | tiny_http | 0.12.0 | crates.io | https://crates.io/crates/tiny_http/0.12.0 |
| 114 | tinystr | 0.8.3 | crates.io | https://crates.io/crates/tinystr/0.8.3 |
| 115 | typenum | 1.20.0 | crates.io | https://crates.io/crates/typenum/1.20.0 |
| 116 | unicode-ident | 1.0.24 | crates.io | https://crates.io/crates/unicode-ident/1.0.24 |
| 117 | unicode-xid | 0.2.6 | crates.io | https://crates.io/crates/unicode-xid/0.2.6 |
| 118 | unsafe-libyaml | 0.2.11 | crates.io | https://crates.io/crates/unsafe-libyaml/0.2.11 |
| 119 | untrusted | 0.9.0 | crates.io | https://crates.io/crates/untrusted/0.9.0 |
| 120 | ureq | 2.12.1 | crates.io | https://crates.io/crates/ureq/2.12.1 |
| 121 | url | 2.5.8 | crates.io | https://crates.io/crates/url/2.5.8 |
| 122 | utf8_iter | 1.0.4 | crates.io | https://crates.io/crates/utf8_iter/1.0.4 |
| 123 | utf8parse | 0.2.2 | crates.io | https://crates.io/crates/utf8parse/0.2.2 |
| 124 | vcpkg | 0.2.15 | crates.io | https://crates.io/crates/vcpkg/0.2.15 |
| 125 | version_check | 0.9.5 | crates.io | https://crates.io/crates/version_check/0.9.5 |
| 126 | wasi | 0.11.1+wasi-snapshot-preview1 | crates.io | https://crates.io/crates/wasi/0.11.1+wasi-snapshot-preview1 |
| 127 | wasip2 | 1.0.3+wasi-0.2.9 | crates.io | https://crates.io/crates/wasip2/1.0.3+wasi-0.2.9 |
| 128 | wasip3 | 0.4.0+wasi-0.3.0-rc-2026-01-06 | crates.io | https://crates.io/crates/wasip3/0.4.0+wasi-0.3.0-rc-2026-01-06 |
| 129 | wasm-encoder | 0.244.0 | crates.io | https://crates.io/crates/wasm-encoder/0.244.0 |
| 130 | wasm-metadata | 0.244.0 | crates.io | https://crates.io/crates/wasm-metadata/0.244.0 |
| 131 | wasmparser | 0.244.0 | crates.io | https://crates.io/crates/wasmparser/0.244.0 |
| 132 | webpki-roots | 0.26.11 | crates.io | https://crates.io/crates/webpki-roots/0.26.11 |
| 133 | webpki-roots | 1.0.7 | crates.io | https://crates.io/crates/webpki-roots/1.0.7 |
| 134 | windows-link | 0.2.1 | crates.io | https://crates.io/crates/windows-link/0.2.1 |
| 135 | windows-sys | 0.52.0 | crates.io | https://crates.io/crates/windows-sys/0.52.0 |
| 136 | windows-sys | 0.59.0 | crates.io | https://crates.io/crates/windows-sys/0.59.0 |
| 137 | windows-sys | 0.61.2 | crates.io | https://crates.io/crates/windows-sys/0.61.2 |
| 138 | windows-targets | 0.52.6 | crates.io | https://crates.io/crates/windows-targets/0.52.6 |
| 139 | windows_aarch64_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_aarch64_gnullvm/0.52.6 |
| 140 | windows_aarch64_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_aarch64_msvc/0.52.6 |
| 141 | windows_i686_gnu | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_gnu/0.52.6 |
| 142 | windows_i686_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_gnullvm/0.52.6 |
| 143 | windows_i686_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_msvc/0.52.6 |
| 144 | windows_x86_64_gnu | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_gnu/0.52.6 |
| 145 | windows_x86_64_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_gnullvm/0.52.6 |
| 146 | windows_x86_64_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_msvc/0.52.6 |
| 147 | wit-bindgen | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen/0.51.0 |
| 148 | wit-bindgen | 0.57.1 | crates.io | https://crates.io/crates/wit-bindgen/0.57.1 |
| 149 | wit-bindgen-core | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-core/0.51.0 |
| 150 | wit-bindgen-rust | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-rust/0.51.0 |
| 151 | wit-bindgen-rust-macro | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-rust-macro/0.51.0 |
| 152 | wit-component | 0.244.0 | crates.io | https://crates.io/crates/wit-component/0.244.0 |
| 153 | wit-parser | 0.244.0 | crates.io | https://crates.io/crates/wit-parser/0.244.0 |
| 154 | writeable | 0.6.3 | crates.io | https://crates.io/crates/writeable/0.6.3 |
| 155 | yoke | 0.8.2 | crates.io | https://crates.io/crates/yoke/0.8.2 |
| 156 | yoke-derive | 0.8.2 | crates.io | https://crates.io/crates/yoke-derive/0.8.2 |
| 157 | zerocopy | 0.8.48 | crates.io | https://crates.io/crates/zerocopy/0.8.48 |
| 158 | zerocopy-derive | 0.8.48 | crates.io | https://crates.io/crates/zerocopy-derive/0.8.48 |
| 159 | zerofrom | 0.1.8 | crates.io | https://crates.io/crates/zerofrom/0.1.8 |
| 160 | zerofrom-derive | 0.1.7 | crates.io | https://crates.io/crates/zerofrom-derive/0.1.7 |
| 161 | zeroize | 1.8.2 | crates.io | https://crates.io/crates/zeroize/1.8.2 |
| 162 | zerotrie | 0.2.4 | crates.io | https://crates.io/crates/zerotrie/0.2.4 |
| 163 | zerovec | 0.11.6 | crates.io | https://crates.io/crates/zerovec/0.11.6 |
| 164 | zerovec-derive | 0.11.3 | crates.io | https://crates.io/crates/zerovec-derive/0.11.3 |
| 165 | zmij | 1.0.21 | crates.io | https://crates.io/crates/zmij/1.0.21 |

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
