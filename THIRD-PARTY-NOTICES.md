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

- Total entries in `Cargo.lock`: 171
- Workspace crates (this repo): 1
- **Third-party dependencies**: 170

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
| 11 | block-buffer | 0.12.1 | crates.io | https://crates.io/crates/block-buffer/0.12.1 |
| 12 | cc | 1.2.62 | crates.io | https://crates.io/crates/cc/1.2.62 |
| 13 | cfg-if | 1.0.4 | crates.io | https://crates.io/crates/cfg-if/1.0.4 |
| 14 | chunked_transfer | 1.5.0 | crates.io | https://crates.io/crates/chunked_transfer/1.5.0 |
| 15 | clap | 4.6.1 | crates.io | https://crates.io/crates/clap/4.6.1 |
| 16 | clap_builder | 4.6.0 | crates.io | https://crates.io/crates/clap_builder/4.6.0 |
| 17 | clap_derive | 4.6.1 | crates.io | https://crates.io/crates/clap_derive/4.6.1 |
| 18 | clap_lex | 1.1.0 | crates.io | https://crates.io/crates/clap_lex/1.1.0 |
| 19 | cmov | 0.5.4 | crates.io | https://crates.io/crates/cmov/0.5.4 |
| 20 | colorchoice | 1.0.5 | crates.io | https://crates.io/crates/colorchoice/1.0.5 |
| 21 | const-oid | 0.10.2 | crates.io | https://crates.io/crates/const-oid/0.10.2 |
| 22 | cpufeatures | 0.3.0 | crates.io | https://crates.io/crates/cpufeatures/0.3.0 |
| 23 | crypto-common | 0.2.2 | crates.io | https://crates.io/crates/crypto-common/0.2.2 |
| 24 | ctutils | 0.4.2 | crates.io | https://crates.io/crates/ctutils/0.4.2 |
| 25 | digest | 0.11.3 | crates.io | https://crates.io/crates/digest/0.11.3 |
| 26 | displaydoc | 0.2.5 | crates.io | https://crates.io/crates/displaydoc/0.2.5 |
| 27 | equivalent | 1.0.2 | crates.io | https://crates.io/crates/equivalent/1.0.2 |
| 28 | errno | 0.3.14 | crates.io | https://crates.io/crates/errno/0.3.14 |
| 29 | fallible-iterator | 0.3.0 | crates.io | https://crates.io/crates/fallible-iterator/0.3.0 |
| 30 | fallible-streaming-iterator | 0.1.9 | crates.io | https://crates.io/crates/fallible-streaming-iterator/0.1.9 |
| 31 | fastrand | 2.4.1 | crates.io | https://crates.io/crates/fastrand/2.4.1 |
| 32 | find-msvc-tools | 0.1.9 | crates.io | https://crates.io/crates/find-msvc-tools/0.1.9 |
| 33 | flume | 0.11.1 | crates.io | https://crates.io/crates/flume/0.11.1 |
| 34 | foldhash | 0.1.5 | crates.io | https://crates.io/crates/foldhash/0.1.5 |
| 35 | form_urlencoded | 1.2.2 | crates.io | https://crates.io/crates/form_urlencoded/1.2.2 |
| 36 | futures-core | 0.3.32 | crates.io | https://crates.io/crates/futures-core/0.3.32 |
| 37 | futures-sink | 0.3.32 | crates.io | https://crates.io/crates/futures-sink/0.3.32 |
| 38 | getrandom | 0.2.17 | crates.io | https://crates.io/crates/getrandom/0.2.17 |
| 39 | getrandom | 0.4.2 | crates.io | https://crates.io/crates/getrandom/0.4.2 |
| 40 | hashbrown | 0.14.5 | crates.io | https://crates.io/crates/hashbrown/0.14.5 |
| 41 | hashbrown | 0.15.5 | crates.io | https://crates.io/crates/hashbrown/0.15.5 |
| 42 | hashbrown | 0.17.1 | crates.io | https://crates.io/crates/hashbrown/0.17.1 |
| 43 | hashlink | 0.9.1 | crates.io | https://crates.io/crates/hashlink/0.9.1 |
| 44 | heck | 0.5.0 | crates.io | https://crates.io/crates/heck/0.5.0 |
| 45 | hmac | 0.13.0 | crates.io | https://crates.io/crates/hmac/0.13.0 |
| 46 | httpdate | 1.0.3 | crates.io | https://crates.io/crates/httpdate/1.0.3 |
| 47 | hybrid-array | 0.4.12 | crates.io | https://crates.io/crates/hybrid-array/0.4.12 |
| 48 | icu_collections | 2.2.0 | crates.io | https://crates.io/crates/icu_collections/2.2.0 |
| 49 | icu_locale_core | 2.2.0 | crates.io | https://crates.io/crates/icu_locale_core/2.2.0 |
| 50 | icu_normalizer | 2.2.0 | crates.io | https://crates.io/crates/icu_normalizer/2.2.0 |
| 51 | icu_normalizer_data | 2.2.0 | crates.io | https://crates.io/crates/icu_normalizer_data/2.2.0 |
| 52 | icu_properties | 2.2.0 | crates.io | https://crates.io/crates/icu_properties/2.2.0 |
| 53 | icu_properties_data | 2.2.0 | crates.io | https://crates.io/crates/icu_properties_data/2.2.0 |
| 54 | icu_provider | 2.2.0 | crates.io | https://crates.io/crates/icu_provider/2.2.0 |
| 55 | id-arena | 2.3.0 | crates.io | https://crates.io/crates/id-arena/2.3.0 |
| 56 | idna | 1.1.0 | crates.io | https://crates.io/crates/idna/1.1.0 |
| 57 | idna_adapter | 1.2.2 | crates.io | https://crates.io/crates/idna_adapter/1.2.2 |
| 58 | if-addrs | 0.15.0 | crates.io | https://crates.io/crates/if-addrs/0.15.0 |
| 59 | indexmap | 2.14.0 | crates.io | https://crates.io/crates/indexmap/2.14.0 |
| 60 | inventory | 0.1.0 | _workspace (this repo)_ | n/a |
| 61 | is_terminal_polyfill | 1.70.2 | crates.io | https://crates.io/crates/is_terminal_polyfill/1.70.2 |
| 62 | itoa | 1.0.18 | crates.io | https://crates.io/crates/itoa/1.0.18 |
| 63 | leb128fmt | 0.1.0 | crates.io | https://crates.io/crates/leb128fmt/0.1.0 |
| 64 | libc | 0.2.186 | crates.io | https://crates.io/crates/libc/0.2.186 |
| 65 | libsqlite3-sys | 0.28.0 | crates.io | https://crates.io/crates/libsqlite3-sys/0.28.0 |
| 66 | linux-raw-sys | 0.12.1 | crates.io | https://crates.io/crates/linux-raw-sys/0.12.1 |
| 67 | litemap | 0.8.2 | crates.io | https://crates.io/crates/litemap/0.8.2 |
| 68 | lock_api | 0.4.14 | crates.io | https://crates.io/crates/lock_api/0.4.14 |
| 69 | log | 0.4.29 | crates.io | https://crates.io/crates/log/0.4.29 |
| 70 | mdns-sd | 0.20.0 | crates.io | https://crates.io/crates/mdns-sd/0.20.0 |
| 71 | memchr | 2.8.0 | crates.io | https://crates.io/crates/memchr/2.8.0 |
| 72 | mio | 1.2.0 | crates.io | https://crates.io/crates/mio/1.2.0 |
| 73 | once_cell | 1.21.4 | crates.io | https://crates.io/crates/once_cell/1.21.4 |
| 74 | once_cell_polyfill | 1.70.2 | crates.io | https://crates.io/crates/once_cell_polyfill/1.70.2 |
| 75 | pbkdf2 | 0.13.0 | crates.io | https://crates.io/crates/pbkdf2/0.13.0 |
| 76 | percent-encoding | 2.3.2 | crates.io | https://crates.io/crates/percent-encoding/2.3.2 |
| 77 | pkg-config | 0.3.33 | crates.io | https://crates.io/crates/pkg-config/0.3.33 |
| 78 | potential_utf | 0.1.5 | crates.io | https://crates.io/crates/potential_utf/0.1.5 |
| 79 | prettyplease | 0.2.37 | crates.io | https://crates.io/crates/prettyplease/0.2.37 |
| 80 | proc-macro2 | 1.0.106 | crates.io | https://crates.io/crates/proc-macro2/1.0.106 |
| 81 | quote | 1.0.45 | crates.io | https://crates.io/crates/quote/1.0.45 |
| 82 | r-efi | 6.0.0 | crates.io | https://crates.io/crates/r-efi/6.0.0 |
| 83 | ring | 0.17.14 | crates.io | https://crates.io/crates/ring/0.17.14 |
| 84 | roxmltree | 0.21.1 | crates.io | https://crates.io/crates/roxmltree/0.21.1 |
| 85 | rusqlite | 0.31.0 | crates.io | https://crates.io/crates/rusqlite/0.31.0 |
| 86 | rustix | 1.1.4 | crates.io | https://crates.io/crates/rustix/1.1.4 |
| 87 | rustls | 0.23.40 | crates.io | https://crates.io/crates/rustls/0.23.40 |
| 88 | rustls-pki-types | 1.14.1 | crates.io | https://crates.io/crates/rustls-pki-types/1.14.1 |
| 89 | rustls-webpki | 0.103.13 | crates.io | https://crates.io/crates/rustls-webpki/0.103.13 |
| 90 | ryu | 1.0.23 | crates.io | https://crates.io/crates/ryu/1.0.23 |
| 91 | scopeguard | 1.2.0 | crates.io | https://crates.io/crates/scopeguard/1.2.0 |
| 92 | semver | 1.0.28 | crates.io | https://crates.io/crates/semver/1.0.28 |
| 93 | serde | 1.0.228 | crates.io | https://crates.io/crates/serde/1.0.228 |
| 94 | serde_core | 1.0.228 | crates.io | https://crates.io/crates/serde_core/1.0.228 |
| 95 | serde_derive | 1.0.228 | crates.io | https://crates.io/crates/serde_derive/1.0.228 |
| 96 | serde_json | 1.0.150 | crates.io | https://crates.io/crates/serde_json/1.0.150 |
| 97 | serde_yaml_ng | 0.10.0 | crates.io | https://crates.io/crates/serde_yaml_ng/0.10.0 |
| 98 | sha2 | 0.11.0 | crates.io | https://crates.io/crates/sha2/0.11.0 |
| 99 | shlex | 1.3.0 | crates.io | https://crates.io/crates/shlex/1.3.0 |
| 100 | smallvec | 1.15.1 | crates.io | https://crates.io/crates/smallvec/1.15.1 |
| 101 | socket-pktinfo | 0.3.2 | crates.io | https://crates.io/crates/socket-pktinfo/0.3.2 |
| 102 | socket2 | 0.6.4 | crates.io | https://crates.io/crates/socket2/0.6.4 |
| 103 | spin | 0.9.8 | crates.io | https://crates.io/crates/spin/0.9.8 |
| 104 | stable_deref_trait | 1.2.1 | crates.io | https://crates.io/crates/stable_deref_trait/1.2.1 |
| 105 | strsim | 0.11.1 | crates.io | https://crates.io/crates/strsim/0.11.1 |
| 106 | subtle | 2.6.1 | crates.io | https://crates.io/crates/subtle/2.6.1 |
| 107 | syn | 2.0.117 | crates.io | https://crates.io/crates/syn/2.0.117 |
| 108 | synstructure | 0.13.2 | crates.io | https://crates.io/crates/synstructure/0.13.2 |
| 109 | tempfile | 3.27.0 | crates.io | https://crates.io/crates/tempfile/3.27.0 |
| 110 | tiny_http | 0.12.0 | crates.io | https://crates.io/crates/tiny_http/0.12.0 |
| 111 | tinystr | 0.8.3 | crates.io | https://crates.io/crates/tinystr/0.8.3 |
| 112 | typenum | 1.20.0 | crates.io | https://crates.io/crates/typenum/1.20.0 |
| 113 | unicode-ident | 1.0.24 | crates.io | https://crates.io/crates/unicode-ident/1.0.24 |
| 114 | unicode-xid | 0.2.6 | crates.io | https://crates.io/crates/unicode-xid/0.2.6 |
| 115 | unsafe-libyaml | 0.2.11 | crates.io | https://crates.io/crates/unsafe-libyaml/0.2.11 |
| 116 | untrusted | 0.9.0 | crates.io | https://crates.io/crates/untrusted/0.9.0 |
| 117 | ureq | 2.12.1 | crates.io | https://crates.io/crates/ureq/2.12.1 |
| 118 | url | 2.5.8 | crates.io | https://crates.io/crates/url/2.5.8 |
| 119 | utf8_iter | 1.0.4 | crates.io | https://crates.io/crates/utf8_iter/1.0.4 |
| 120 | utf8parse | 0.2.2 | crates.io | https://crates.io/crates/utf8parse/0.2.2 |
| 121 | vcpkg | 0.2.15 | crates.io | https://crates.io/crates/vcpkg/0.2.15 |
| 122 | version_check | 0.9.5 | crates.io | https://crates.io/crates/version_check/0.9.5 |
| 123 | wasi | 0.11.1+wasi-snapshot-preview1 | crates.io | https://crates.io/crates/wasi/0.11.1+wasi-snapshot-preview1 |
| 124 | wasip2 | 1.0.3+wasi-0.2.9 | crates.io | https://crates.io/crates/wasip2/1.0.3+wasi-0.2.9 |
| 125 | wasip3 | 0.4.0+wasi-0.3.0-rc-2026-01-06 | crates.io | https://crates.io/crates/wasip3/0.4.0+wasi-0.3.0-rc-2026-01-06 |
| 126 | wasm-encoder | 0.244.0 | crates.io | https://crates.io/crates/wasm-encoder/0.244.0 |
| 127 | wasm-metadata | 0.244.0 | crates.io | https://crates.io/crates/wasm-metadata/0.244.0 |
| 128 | wasmparser | 0.244.0 | crates.io | https://crates.io/crates/wasmparser/0.244.0 |
| 129 | webpki-roots | 0.26.11 | crates.io | https://crates.io/crates/webpki-roots/0.26.11 |
| 130 | webpki-roots | 1.0.7 | crates.io | https://crates.io/crates/webpki-roots/1.0.7 |
| 131 | windows-link | 0.2.1 | crates.io | https://crates.io/crates/windows-link/0.2.1 |
| 132 | windows-sys | 0.52.0 | crates.io | https://crates.io/crates/windows-sys/0.52.0 |
| 133 | windows-sys | 0.60.2 | crates.io | https://crates.io/crates/windows-sys/0.60.2 |
| 134 | windows-sys | 0.61.2 | crates.io | https://crates.io/crates/windows-sys/0.61.2 |
| 135 | windows-targets | 0.52.6 | crates.io | https://crates.io/crates/windows-targets/0.52.6 |
| 136 | windows-targets | 0.53.5 | crates.io | https://crates.io/crates/windows-targets/0.53.5 |
| 137 | windows_aarch64_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_aarch64_gnullvm/0.52.6 |
| 138 | windows_aarch64_gnullvm | 0.53.1 | crates.io | https://crates.io/crates/windows_aarch64_gnullvm/0.53.1 |
| 139 | windows_aarch64_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_aarch64_msvc/0.52.6 |
| 140 | windows_aarch64_msvc | 0.53.1 | crates.io | https://crates.io/crates/windows_aarch64_msvc/0.53.1 |
| 141 | windows_i686_gnu | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_gnu/0.52.6 |
| 142 | windows_i686_gnu | 0.53.1 | crates.io | https://crates.io/crates/windows_i686_gnu/0.53.1 |
| 143 | windows_i686_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_gnullvm/0.52.6 |
| 144 | windows_i686_gnullvm | 0.53.1 | crates.io | https://crates.io/crates/windows_i686_gnullvm/0.53.1 |
| 145 | windows_i686_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_msvc/0.52.6 |
| 146 | windows_i686_msvc | 0.53.1 | crates.io | https://crates.io/crates/windows_i686_msvc/0.53.1 |
| 147 | windows_x86_64_gnu | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_gnu/0.52.6 |
| 148 | windows_x86_64_gnu | 0.53.1 | crates.io | https://crates.io/crates/windows_x86_64_gnu/0.53.1 |
| 149 | windows_x86_64_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_gnullvm/0.52.6 |
| 150 | windows_x86_64_gnullvm | 0.53.1 | crates.io | https://crates.io/crates/windows_x86_64_gnullvm/0.53.1 |
| 151 | windows_x86_64_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_msvc/0.52.6 |
| 152 | windows_x86_64_msvc | 0.53.1 | crates.io | https://crates.io/crates/windows_x86_64_msvc/0.53.1 |
| 153 | wit-bindgen | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen/0.51.0 |
| 154 | wit-bindgen | 0.57.1 | crates.io | https://crates.io/crates/wit-bindgen/0.57.1 |
| 155 | wit-bindgen-core | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-core/0.51.0 |
| 156 | wit-bindgen-rust | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-rust/0.51.0 |
| 157 | wit-bindgen-rust-macro | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-rust-macro/0.51.0 |
| 158 | wit-component | 0.244.0 | crates.io | https://crates.io/crates/wit-component/0.244.0 |
| 159 | wit-parser | 0.244.0 | crates.io | https://crates.io/crates/wit-parser/0.244.0 |
| 160 | writeable | 0.6.3 | crates.io | https://crates.io/crates/writeable/0.6.3 |
| 161 | yoke | 0.8.2 | crates.io | https://crates.io/crates/yoke/0.8.2 |
| 162 | yoke-derive | 0.8.2 | crates.io | https://crates.io/crates/yoke-derive/0.8.2 |
| 163 | zerocopy | 0.8.48 | crates.io | https://crates.io/crates/zerocopy/0.8.48 |
| 164 | zerocopy-derive | 0.8.48 | crates.io | https://crates.io/crates/zerocopy-derive/0.8.48 |
| 165 | zerofrom | 0.1.8 | crates.io | https://crates.io/crates/zerofrom/0.1.8 |
| 166 | zerofrom-derive | 0.1.7 | crates.io | https://crates.io/crates/zerofrom-derive/0.1.7 |
| 167 | zeroize | 1.8.2 | crates.io | https://crates.io/crates/zeroize/1.8.2 |
| 168 | zerotrie | 0.2.4 | crates.io | https://crates.io/crates/zerotrie/0.2.4 |
| 169 | zerovec | 0.11.6 | crates.io | https://crates.io/crates/zerovec/0.11.6 |
| 170 | zerovec-derive | 0.11.3 | crates.io | https://crates.io/crates/zerovec-derive/0.11.3 |
| 171 | zmij | 1.0.21 | crates.io | https://crates.io/crates/zmij/1.0.21 |

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
