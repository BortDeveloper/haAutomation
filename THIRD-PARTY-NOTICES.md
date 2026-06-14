# Third-Party Open Source Notices

This file lists every open source dependency of the `home-inventory`
workspace, with version and source. It is **automatically generated**
and **drift-checked by CI** on every push.

- **Source of truth**: `home-inventory/Cargo.lock` (workspace + transitive)
- **Generator**: `home-inventory/scripts/generate-notices.sh` (parses
  `Cargo.lock` directly; no network, no cargo required)
- **Drift check**: `.github/workflows/security.yml`, job
  `license-notices` — re-runs the generator and `diff`s against
  this committed file. CI fails on any divergence.
- **License enforcement**: `home-inventory/deny.toml` (`cargo deny check
  licenses`) restricts the accepted SPDX set to permissive licenses
  (MIT, Apache-2.0, BSD-2/3-Clause, ISC, Unicode-DFS-2016, CC0-1.0,
  Zlib). Copyleft is denied. License compliance is therefore enforced
  by `cargo deny` even though the per-crate SPDX string is not
  inlined into this file (Cargo.lock does not record it).

If you add, remove, or update a dependency:

```
bash home-inventory/scripts/generate-notices.sh > THIRD-PARTY-NOTICES.md
```

…and commit the regenerated file alongside your `Cargo.lock` change.
CI will block the merge if you forget.

## Summary

- Total entries in `Cargo.lock`: 179
- Workspace crates (this repo): 1
- **Third-party dependencies**: 178

## Accepted licenses

The following SPDX identifiers are accepted by the project (see
`home-inventory/deny.toml` and `home-inventory/about.toml`):

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
| 1 | anstream | 1.0.0 | crates.io | https://crates.io/crates/anstream/1.0.0 |
| 2 | anstyle | 1.0.14 | crates.io | https://crates.io/crates/anstyle/1.0.14 |
| 3 | anstyle-parse | 1.0.0 | crates.io | https://crates.io/crates/anstyle-parse/1.0.0 |
| 4 | anstyle-query | 1.1.5 | crates.io | https://crates.io/crates/anstyle-query/1.1.5 |
| 5 | anstyle-wincon | 3.0.11 | crates.io | https://crates.io/crates/anstyle-wincon/3.0.11 |
| 6 | anyhow | 1.0.102 | crates.io | https://crates.io/crates/anyhow/1.0.102 |
| 7 | ascii | 1.1.0 | crates.io | https://crates.io/crates/ascii/1.1.0 |
| 8 | base64 | 0.22.1 | crates.io | https://crates.io/crates/base64/0.22.1 |
| 9 | bitflags | 2.11.1 | crates.io | https://crates.io/crates/bitflags/2.11.1 |
| 10 | block-buffer | 0.12.1 | crates.io | https://crates.io/crates/block-buffer/0.12.1 |
| 11 | bumpalo | 3.20.3 | crates.io | https://crates.io/crates/bumpalo/3.20.3 |
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
| 35 | foldhash | 0.2.0 | crates.io | https://crates.io/crates/foldhash/0.2.0 |
| 36 | form_urlencoded | 1.2.2 | crates.io | https://crates.io/crates/form_urlencoded/1.2.2 |
| 37 | futures-core | 0.3.32 | crates.io | https://crates.io/crates/futures-core/0.3.32 |
| 38 | futures-sink | 0.3.32 | crates.io | https://crates.io/crates/futures-sink/0.3.32 |
| 39 | getrandom | 0.2.17 | crates.io | https://crates.io/crates/getrandom/0.2.17 |
| 40 | getrandom | 0.4.2 | crates.io | https://crates.io/crates/getrandom/0.4.2 |
| 41 | hashbrown | 0.15.5 | crates.io | https://crates.io/crates/hashbrown/0.15.5 |
| 42 | hashbrown | 0.16.1 | crates.io | https://crates.io/crates/hashbrown/0.16.1 |
| 43 | hashbrown | 0.17.1 | crates.io | https://crates.io/crates/hashbrown/0.17.1 |
| 44 | hashlink | 0.12.0 | crates.io | https://crates.io/crates/hashlink/0.12.0 |
| 45 | heck | 0.5.0 | crates.io | https://crates.io/crates/heck/0.5.0 |
| 46 | hmac | 0.13.0 | crates.io | https://crates.io/crates/hmac/0.13.0 |
| 47 | home-inventory | 0.1.0 | _workspace (this repo)_ | n/a |
| 48 | httpdate | 1.0.3 | crates.io | https://crates.io/crates/httpdate/1.0.3 |
| 49 | hybrid-array | 0.4.12 | crates.io | https://crates.io/crates/hybrid-array/0.4.12 |
| 50 | icu_collections | 2.2.0 | crates.io | https://crates.io/crates/icu_collections/2.2.0 |
| 51 | icu_locale_core | 2.2.0 | crates.io | https://crates.io/crates/icu_locale_core/2.2.0 |
| 52 | icu_normalizer | 2.2.0 | crates.io | https://crates.io/crates/icu_normalizer/2.2.0 |
| 53 | icu_normalizer_data | 2.2.0 | crates.io | https://crates.io/crates/icu_normalizer_data/2.2.0 |
| 54 | icu_properties | 2.2.0 | crates.io | https://crates.io/crates/icu_properties/2.2.0 |
| 55 | icu_properties_data | 2.2.0 | crates.io | https://crates.io/crates/icu_properties_data/2.2.0 |
| 56 | icu_provider | 2.2.0 | crates.io | https://crates.io/crates/icu_provider/2.2.0 |
| 57 | id-arena | 2.3.0 | crates.io | https://crates.io/crates/id-arena/2.3.0 |
| 58 | idna | 1.1.0 | crates.io | https://crates.io/crates/idna/1.1.0 |
| 59 | idna_adapter | 1.2.2 | crates.io | https://crates.io/crates/idna_adapter/1.2.2 |
| 60 | if-addrs | 0.15.0 | crates.io | https://crates.io/crates/if-addrs/0.15.0 |
| 61 | indexmap | 2.14.0 | crates.io | https://crates.io/crates/indexmap/2.14.0 |
| 62 | is_terminal_polyfill | 1.70.2 | crates.io | https://crates.io/crates/is_terminal_polyfill/1.70.2 |
| 63 | itoa | 1.0.18 | crates.io | https://crates.io/crates/itoa/1.0.18 |
| 64 | js-sys | 0.3.102 | crates.io | https://crates.io/crates/js-sys/0.3.102 |
| 65 | leb128fmt | 0.1.0 | crates.io | https://crates.io/crates/leb128fmt/0.1.0 |
| 66 | libc | 0.2.186 | crates.io | https://crates.io/crates/libc/0.2.186 |
| 67 | libsqlite3-sys | 0.38.1 | crates.io | https://crates.io/crates/libsqlite3-sys/0.38.1 |
| 68 | linux-raw-sys | 0.12.1 | crates.io | https://crates.io/crates/linux-raw-sys/0.12.1 |
| 69 | litemap | 0.8.2 | crates.io | https://crates.io/crates/litemap/0.8.2 |
| 70 | lock_api | 0.4.14 | crates.io | https://crates.io/crates/lock_api/0.4.14 |
| 71 | log | 0.4.29 | crates.io | https://crates.io/crates/log/0.4.29 |
| 72 | mdns-sd | 0.20.0 | crates.io | https://crates.io/crates/mdns-sd/0.20.0 |
| 73 | memchr | 2.8.0 | crates.io | https://crates.io/crates/memchr/2.8.0 |
| 74 | mio | 1.2.0 | crates.io | https://crates.io/crates/mio/1.2.0 |
| 75 | once_cell | 1.21.4 | crates.io | https://crates.io/crates/once_cell/1.21.4 |
| 76 | once_cell_polyfill | 1.70.2 | crates.io | https://crates.io/crates/once_cell_polyfill/1.70.2 |
| 77 | pbkdf2 | 0.13.0 | crates.io | https://crates.io/crates/pbkdf2/0.13.0 |
| 78 | percent-encoding | 2.3.2 | crates.io | https://crates.io/crates/percent-encoding/2.3.2 |
| 79 | pkg-config | 0.3.33 | crates.io | https://crates.io/crates/pkg-config/0.3.33 |
| 80 | potential_utf | 0.1.5 | crates.io | https://crates.io/crates/potential_utf/0.1.5 |
| 81 | prettyplease | 0.2.37 | crates.io | https://crates.io/crates/prettyplease/0.2.37 |
| 82 | proc-macro2 | 1.0.106 | crates.io | https://crates.io/crates/proc-macro2/1.0.106 |
| 83 | quote | 1.0.45 | crates.io | https://crates.io/crates/quote/1.0.45 |
| 84 | r-efi | 6.0.0 | crates.io | https://crates.io/crates/r-efi/6.0.0 |
| 85 | ring | 0.17.14 | crates.io | https://crates.io/crates/ring/0.17.14 |
| 86 | roxmltree | 0.21.1 | crates.io | https://crates.io/crates/roxmltree/0.21.1 |
| 87 | rsqlite-vfs | 0.1.1 | crates.io | https://crates.io/crates/rsqlite-vfs/0.1.1 |
| 88 | rusqlite | 0.40.1 | crates.io | https://crates.io/crates/rusqlite/0.40.1 |
| 89 | rustix | 1.1.4 | crates.io | https://crates.io/crates/rustix/1.1.4 |
| 90 | rustls | 0.23.40 | crates.io | https://crates.io/crates/rustls/0.23.40 |
| 91 | rustls-pki-types | 1.14.1 | crates.io | https://crates.io/crates/rustls-pki-types/1.14.1 |
| 92 | rustls-webpki | 0.103.13 | crates.io | https://crates.io/crates/rustls-webpki/0.103.13 |
| 93 | rustversion | 1.0.22 | crates.io | https://crates.io/crates/rustversion/1.0.22 |
| 94 | ryu | 1.0.23 | crates.io | https://crates.io/crates/ryu/1.0.23 |
| 95 | scopeguard | 1.2.0 | crates.io | https://crates.io/crates/scopeguard/1.2.0 |
| 96 | semver | 1.0.28 | crates.io | https://crates.io/crates/semver/1.0.28 |
| 97 | serde | 1.0.228 | crates.io | https://crates.io/crates/serde/1.0.228 |
| 98 | serde_core | 1.0.228 | crates.io | https://crates.io/crates/serde_core/1.0.228 |
| 99 | serde_derive | 1.0.228 | crates.io | https://crates.io/crates/serde_derive/1.0.228 |
| 100 | serde_json | 1.0.150 | crates.io | https://crates.io/crates/serde_json/1.0.150 |
| 101 | serde_yaml_ng | 0.10.0 | crates.io | https://crates.io/crates/serde_yaml_ng/0.10.0 |
| 102 | sha2 | 0.11.0 | crates.io | https://crates.io/crates/sha2/0.11.0 |
| 103 | shlex | 1.3.0 | crates.io | https://crates.io/crates/shlex/1.3.0 |
| 104 | smallvec | 1.15.1 | crates.io | https://crates.io/crates/smallvec/1.15.1 |
| 105 | socket-pktinfo | 0.3.2 | crates.io | https://crates.io/crates/socket-pktinfo/0.3.2 |
| 106 | socket2 | 0.6.4 | crates.io | https://crates.io/crates/socket2/0.6.4 |
| 107 | spin | 0.9.8 | crates.io | https://crates.io/crates/spin/0.9.8 |
| 108 | sqlite-wasm-rs | 0.5.5 | crates.io | https://crates.io/crates/sqlite-wasm-rs/0.5.5 |
| 109 | stable_deref_trait | 1.2.1 | crates.io | https://crates.io/crates/stable_deref_trait/1.2.1 |
| 110 | strsim | 0.11.1 | crates.io | https://crates.io/crates/strsim/0.11.1 |
| 111 | subtle | 2.6.1 | crates.io | https://crates.io/crates/subtle/2.6.1 |
| 112 | syn | 2.0.117 | crates.io | https://crates.io/crates/syn/2.0.117 |
| 113 | synstructure | 0.13.2 | crates.io | https://crates.io/crates/synstructure/0.13.2 |
| 114 | tempfile | 3.27.0 | crates.io | https://crates.io/crates/tempfile/3.27.0 |
| 115 | thiserror | 2.0.18 | crates.io | https://crates.io/crates/thiserror/2.0.18 |
| 116 | thiserror-impl | 2.0.18 | crates.io | https://crates.io/crates/thiserror-impl/2.0.18 |
| 117 | tiny_http | 0.12.0 | crates.io | https://crates.io/crates/tiny_http/0.12.0 |
| 118 | tinystr | 0.8.3 | crates.io | https://crates.io/crates/tinystr/0.8.3 |
| 119 | typenum | 1.20.0 | crates.io | https://crates.io/crates/typenum/1.20.0 |
| 120 | unicode-ident | 1.0.24 | crates.io | https://crates.io/crates/unicode-ident/1.0.24 |
| 121 | unicode-xid | 0.2.6 | crates.io | https://crates.io/crates/unicode-xid/0.2.6 |
| 122 | unsafe-libyaml | 0.2.11 | crates.io | https://crates.io/crates/unsafe-libyaml/0.2.11 |
| 123 | untrusted | 0.9.0 | crates.io | https://crates.io/crates/untrusted/0.9.0 |
| 124 | ureq | 2.12.1 | crates.io | https://crates.io/crates/ureq/2.12.1 |
| 125 | url | 2.5.8 | crates.io | https://crates.io/crates/url/2.5.8 |
| 126 | utf8_iter | 1.0.4 | crates.io | https://crates.io/crates/utf8_iter/1.0.4 |
| 127 | utf8parse | 0.2.2 | crates.io | https://crates.io/crates/utf8parse/0.2.2 |
| 128 | vcpkg | 0.2.15 | crates.io | https://crates.io/crates/vcpkg/0.2.15 |
| 129 | wasi | 0.11.1+wasi-snapshot-preview1 | crates.io | https://crates.io/crates/wasi/0.11.1+wasi-snapshot-preview1 |
| 130 | wasip2 | 1.0.3+wasi-0.2.9 | crates.io | https://crates.io/crates/wasip2/1.0.3+wasi-0.2.9 |
| 131 | wasip3 | 0.4.0+wasi-0.3.0-rc-2026-01-06 | crates.io | https://crates.io/crates/wasip3/0.4.0+wasi-0.3.0-rc-2026-01-06 |
| 132 | wasm-bindgen | 0.2.125 | crates.io | https://crates.io/crates/wasm-bindgen/0.2.125 |
| 133 | wasm-bindgen-macro | 0.2.125 | crates.io | https://crates.io/crates/wasm-bindgen-macro/0.2.125 |
| 134 | wasm-bindgen-macro-support | 0.2.125 | crates.io | https://crates.io/crates/wasm-bindgen-macro-support/0.2.125 |
| 135 | wasm-bindgen-shared | 0.2.125 | crates.io | https://crates.io/crates/wasm-bindgen-shared/0.2.125 |
| 136 | wasm-encoder | 0.244.0 | crates.io | https://crates.io/crates/wasm-encoder/0.244.0 |
| 137 | wasm-metadata | 0.244.0 | crates.io | https://crates.io/crates/wasm-metadata/0.244.0 |
| 138 | wasmparser | 0.244.0 | crates.io | https://crates.io/crates/wasmparser/0.244.0 |
| 139 | webpki-roots | 0.26.11 | crates.io | https://crates.io/crates/webpki-roots/0.26.11 |
| 140 | webpki-roots | 1.0.7 | crates.io | https://crates.io/crates/webpki-roots/1.0.7 |
| 141 | windows-link | 0.2.1 | crates.io | https://crates.io/crates/windows-link/0.2.1 |
| 142 | windows-sys | 0.52.0 | crates.io | https://crates.io/crates/windows-sys/0.52.0 |
| 143 | windows-sys | 0.60.2 | crates.io | https://crates.io/crates/windows-sys/0.60.2 |
| 144 | windows-sys | 0.61.2 | crates.io | https://crates.io/crates/windows-sys/0.61.2 |
| 145 | windows-targets | 0.52.6 | crates.io | https://crates.io/crates/windows-targets/0.52.6 |
| 146 | windows-targets | 0.53.5 | crates.io | https://crates.io/crates/windows-targets/0.53.5 |
| 147 | windows_aarch64_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_aarch64_gnullvm/0.52.6 |
| 148 | windows_aarch64_gnullvm | 0.53.1 | crates.io | https://crates.io/crates/windows_aarch64_gnullvm/0.53.1 |
| 149 | windows_aarch64_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_aarch64_msvc/0.52.6 |
| 150 | windows_aarch64_msvc | 0.53.1 | crates.io | https://crates.io/crates/windows_aarch64_msvc/0.53.1 |
| 151 | windows_i686_gnu | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_gnu/0.52.6 |
| 152 | windows_i686_gnu | 0.53.1 | crates.io | https://crates.io/crates/windows_i686_gnu/0.53.1 |
| 153 | windows_i686_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_gnullvm/0.52.6 |
| 154 | windows_i686_gnullvm | 0.53.1 | crates.io | https://crates.io/crates/windows_i686_gnullvm/0.53.1 |
| 155 | windows_i686_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_i686_msvc/0.52.6 |
| 156 | windows_i686_msvc | 0.53.1 | crates.io | https://crates.io/crates/windows_i686_msvc/0.53.1 |
| 157 | windows_x86_64_gnu | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_gnu/0.52.6 |
| 158 | windows_x86_64_gnu | 0.53.1 | crates.io | https://crates.io/crates/windows_x86_64_gnu/0.53.1 |
| 159 | windows_x86_64_gnullvm | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_gnullvm/0.52.6 |
| 160 | windows_x86_64_gnullvm | 0.53.1 | crates.io | https://crates.io/crates/windows_x86_64_gnullvm/0.53.1 |
| 161 | windows_x86_64_msvc | 0.52.6 | crates.io | https://crates.io/crates/windows_x86_64_msvc/0.52.6 |
| 162 | windows_x86_64_msvc | 0.53.1 | crates.io | https://crates.io/crates/windows_x86_64_msvc/0.53.1 |
| 163 | wit-bindgen | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen/0.51.0 |
| 164 | wit-bindgen | 0.57.1 | crates.io | https://crates.io/crates/wit-bindgen/0.57.1 |
| 165 | wit-bindgen-core | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-core/0.51.0 |
| 166 | wit-bindgen-rust | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-rust/0.51.0 |
| 167 | wit-bindgen-rust-macro | 0.51.0 | crates.io | https://crates.io/crates/wit-bindgen-rust-macro/0.51.0 |
| 168 | wit-component | 0.244.0 | crates.io | https://crates.io/crates/wit-component/0.244.0 |
| 169 | wit-parser | 0.244.0 | crates.io | https://crates.io/crates/wit-parser/0.244.0 |
| 170 | writeable | 0.6.3 | crates.io | https://crates.io/crates/writeable/0.6.3 |
| 171 | yoke | 0.8.2 | crates.io | https://crates.io/crates/yoke/0.8.2 |
| 172 | yoke-derive | 0.8.2 | crates.io | https://crates.io/crates/yoke-derive/0.8.2 |
| 173 | zerofrom | 0.1.8 | crates.io | https://crates.io/crates/zerofrom/0.1.8 |
| 174 | zerofrom-derive | 0.1.7 | crates.io | https://crates.io/crates/zerofrom-derive/0.1.7 |
| 175 | zeroize | 1.8.2 | crates.io | https://crates.io/crates/zeroize/1.8.2 |
| 176 | zerotrie | 0.2.4 | crates.io | https://crates.io/crates/zerotrie/0.2.4 |
| 177 | zerovec | 0.11.6 | crates.io | https://crates.io/crates/zerovec/0.11.6 |
| 178 | zerovec-derive | 0.11.3 | crates.io | https://crates.io/crates/zerovec-derive/0.11.3 |
| 179 | zmij | 1.0.21 | crates.io | https://crates.io/crates/zmij/1.0.21 |

---

For each dependency, the canonical license text is published on its
crates.io page (linked above) and bundled with the crate source in
`~/.cargo/registry/src/`. To see the SPDX license string for a
specific crate locally:

```
cargo metadata --format-version 1 --manifest-path home-inventory/Cargo.toml \
    | jq -r '.packages[] | "\(.name) \(.version) \(.license // .license_file // "UNKNOWN")"' \
    | sort -u
```

CI runs `cargo deny check licenses` on every push, so any crate
without an accepted license string in its `Cargo.toml` will block
the build.
