# Third-party components

Everything the app ships, with its licence. The app makes no network
connections and bundles no codec: audio and video are decoded and
encoded by macOS (AVFoundation).

## Model

- **TIGER-DnR, music branch** (Xu, Li, Chen, Hu; ICLR 2025;
  https://huggingface.co/JusperLee/TIGER-DnR), weights Apache-2.0
  (`licenses/TIGER-DnR-LICENSE-APACHE.txt`), converted to ONNX with
  `charon-audio/tools/export/export_tiger.py`; the changes are listed in
  `licenses/TIGER-DnR-NOTICE.txt`. Trained on Divide and Remaster v1,
  whose music and effects include clips licensed for non-commercial use
  only; see "Model and training data" in the README and decision 0012.

## Runtime and engine

- **charon-audio**, MIT OR Apache-2.0 (`licenses/charon-audio-*`).
- **ONNX Runtime 1.28**, MIT (`licenses/onnxruntime-LICENSE.txt`), with
  its third-party notices (`licenses/onnxruntime-ThirdPartyNotices.txt`).
  Statically linked through the `ort` crate.

## Fonts (bundled by egui)

Ubuntu Light (Ubuntu Font Licence, `licenses/font-UFL.txt`), Noto Emoji
(SIL OFL 1.1, `licenses/font-OFL.txt`), Hack (`licenses/font-Hack.txt`),
emoji icon font (MIT, `licenses/font-emoji-icon-MIT.txt`).

## Rust crates

Generated with `cargo tree -e normal` for the macOS build. Where a crate
offers a choice, the permissive option applies (`self_cell`: Apache-2.0).

| crate | licence |
|---|---|
| accesskit v0.24.1 | MIT OR Apache-2.0 |
| accesskit_consumer v0.38.0 | MIT OR Apache-2.0 |
| accesskit_macos v0.26.3 | MIT OR Apache-2.0 |
| accesskit_winit v0.32.2 | Apache-2.0 |
| adler2 v2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| ahash v0.8.12 | MIT OR Apache-2.0 |
| aho-corasick v1.1.5 | Unlicense OR MIT |
| anstream v1.0.0 | MIT OR Apache-2.0 |
| anstyle v1.0.14 | MIT OR Apache-2.0 |
| anstyle-parse v1.0.0 | MIT OR Apache-2.0 |
| anstyle-query v1.1.5 | MIT OR Apache-2.0 |
| anyhow v1.0.104 | MIT OR Apache-2.0 |
| arboard v3.6.1 | MIT OR Apache-2.0 |
| arrayvec v0.7.8 | MIT OR Apache-2.0 |
| bit-set v0.10.0 | Apache-2.0 OR MIT |
| bit-vec v0.9.1 | Apache-2.0 OR MIT |
| bitflags v1.3.2 | MIT/Apache-2.0 |
| bitflags v2.13.2 | MIT OR Apache-2.0 |
| block-buffer v0.10.4 | MIT OR Apache-2.0 |
| block2 v0.5.1 | MIT |
| block2 v0.6.2 | MIT |
| bytemuck v1.25.2 | Zlib OR Apache-2.0 OR MIT |
| bytemuck_derive v1.12.1 | Zlib OR Apache-2.0 OR MIT |
| byteorder v1.5.0 | Unlicense OR MIT |
| byteorder-lite v0.1.0 | Unlicense OR MIT |
| cfg-if v1.0.5 | MIT OR Apache-2.0 |
| charon-audio v0.1.1 | MIT OR Apache-2.0 |
| codespan-reporting v0.13.1 | Apache-2.0 |
| color v0.3.3 | Apache-2.0 OR MIT |
| colorchoice v1.0.5 | MIT OR Apache-2.0 |
| console v0.15.11 | MIT |
| core-foundation v0.9.4 | MIT OR Apache-2.0 |
| core-foundation-sys v0.8.7 | MIT OR Apache-2.0 |
| core-graphics v0.23.2 | MIT OR Apache-2.0 |
| core-graphics-types v0.1.3 | MIT OR Apache-2.0 |
| coreaudio-rs v0.14.2 | MIT/Apache-2.0 |
| cpal v0.18.2 | Apache-2.0 |
| cpufeatures v0.2.17 | MIT OR Apache-2.0 |
| crc v3.4.0 | MIT OR Apache-2.0 |
| crc-catalog v2.5.0 | MIT OR Apache-2.0 |
| crc32fast v1.5.2 | MIT OR Apache-2.0 |
| crossbeam-channel v0.5.17 | MIT OR Apache-2.0 |
| crossbeam-deque v0.8.8 | MIT OR Apache-2.0 |
| crossbeam-epoch v0.9.21 | MIT OR Apache-2.0 |
| crossbeam-utils v0.8.23 | MIT OR Apache-2.0 |
| crypto-common v0.1.7 | MIT OR Apache-2.0 |
| cursor-icon v1.2.0 | MIT OR Apache-2.0 OR Zlib |
| dasp_sample v0.11.0 | MIT OR Apache-2.0 |
| digest v0.10.7 | MIT OR Apache-2.0 |
| dispatch v0.2.0 | MIT |
| dispatch2 v0.3.1 | Zlib OR Apache-2.0 OR MIT |
| displaydoc v0.2.7 | MIT OR Apache-2.0 |
| document-features v0.2.12 | MIT OR Apache-2.0 |
| dpi v0.1.2 | Apache-2.0 AND MIT |
| ecolor v0.36.2 | MIT OR Apache-2.0 |
| eframe v0.36.2 | MIT OR Apache-2.0 |
| egui v0.36.2 | MIT OR Apache-2.0 |
| egui-wgpu v0.36.2 | MIT OR Apache-2.0 |
| egui-winit v0.36.2 | MIT OR Apache-2.0 |
| either v1.18.0 | MIT OR Apache-2.0 |
| emath v0.36.2 | MIT OR Apache-2.0 |
| env_filter v2.0.0 | MIT OR Apache-2.0 |
| env_logger v0.11.11 | MIT OR Apache-2.0 |
| epaint v0.36.2 | MIT OR Apache-2.0 |
| epaint_default_fonts v0.36.2 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 |
| equivalent v1.0.2 | Apache-2.0 OR MIT |
| euclid v0.22.14 | MIT OR Apache-2.0 |
| fax v0.2.7 | MIT |
| fdeflate v0.3.7 | MIT OR Apache-2.0 |
| fearless_simd v0.4.1 | Apache-2.0 OR MIT |
| flacenc v0.5.1 | Apache-2.0 |
| flate2 v1.1.10 | MIT OR Apache-2.0 |
| foldhash v0.2.0 | Zlib |
| font-types v0.12.5 | MIT OR Apache-2.0 |
| foreign-types v0.5.0 | MIT/Apache-2.0 |
| foreign-types-macros v0.2.4 | MIT/Apache-2.0 |
| foreign-types-shared v0.3.1 | MIT/Apache-2.0 |
| form_urlencoded v1.2.2 | MIT OR Apache-2.0 |
| generic-array v0.14.7 | MIT |
| guillotiere v0.7.0 | MIT/Apache-2.0 |
| half v2.7.1 | MIT OR Apache-2.0 |
| harfrust v0.12.0 | MIT |
| hash32 v0.3.1 | MIT OR Apache-2.0 |
| hashbrown v0.16.1 | MIT OR Apache-2.0 |
| hashbrown v0.17.1 | MIT OR Apache-2.0 |
| heapless v0.8.0 | MIT OR Apache-2.0 |
| hound v3.5.1 | Apache-2.0 |
| icu_collections v2.3.0 | Unicode-3.0 |
| icu_locale_core v2.3.0 | Unicode-3.0 |
| icu_normalizer v2.3.0 | Unicode-3.0 |
| icu_normalizer_data v2.3.0 | Unicode-3.0 |
| icu_properties v2.3.0 | Unicode-3.0 |
| icu_properties_data v2.3.0 | Unicode-3.0 |
| icu_provider v2.3.1 | Unicode-3.0 |
| idna v1.1.0 | MIT OR Apache-2.0 |
| idna_adapter v1.2.2 | Apache-2.0 OR MIT |
| image v0.25.10 | MIT OR Apache-2.0 |
| indexmap v2.14.2 | Apache-2.0 OR MIT |
| indicatif v0.17.11 | MIT |
| is_terminal_polyfill v1.70.2 | MIT OR Apache-2.0 |
| itertools v0.15.0 | MIT OR Apache-2.0 |
| itoa v1.0.18 | MIT OR Apache-2.0 |
| jiff v0.2.37 | Unlicense OR MIT |
| jiff-core v0.1.1 | Unlicense OR MIT |
| kurbo v0.13.1 | Apache-2.0 OR MIT |
| libc v0.2.189 | MIT OR Apache-2.0 |
| libloading v0.8.9 | ISC |
| libm v0.2.16 | MIT |
| linebender_resource_handle v0.1.1 | Apache-2.0 OR MIT |
| litemap v0.8.3 | Unicode-3.0 |
| litrs v1.0.0 | MIT OR Apache-2.0 |
| lock_api v0.4.14 | MIT OR Apache-2.0 |
| log v0.4.34 | MIT OR Apache-2.0 |
| mach2 v0.6.0 | BSD-2-Clause OR MIT OR Apache-2.0 |
| matrixmultiply v0.3.11 | MIT/Apache-2.0 |
| md-5 v0.10.6 | MIT OR Apache-2.0 |
| memchr v2.8.3 | Unlicense OR MIT |
| miniz_oxide v0.8.9 | MIT OR Zlib OR Apache-2.0 |
| miniz_oxide v0.9.1 | MIT OR Zlib OR Apache-2.0 |
| moxcms v0.8.1 | BSD-3-Clause OR Apache-2.0 |
| naga v30.0.1 | MIT OR Apache-2.0 |
| naga-types v30.0.1 | MIT OR Apache-2.0 |
| ndarray v0.15.6 | MIT OR Apache-2.0 |
| nohash-hasher v0.2.0 | Apache-2.0 OR MIT |
| num-complex v0.4.6 | MIT OR Apache-2.0 |
| num-integer v0.1.47 | MIT OR Apache-2.0 |
| num-traits v0.2.19 | MIT OR Apache-2.0 |
| number_prefix v0.4.0 | MIT |
| objc-sys v0.3.5 | MIT |
| objc2 v0.5.2 | MIT |
| objc2 v0.6.4 | MIT |
| objc2-app-kit v0.2.2 | MIT |
| objc2-app-kit v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-audio-toolbox v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-av-foundation v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-avf-audio v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-audio v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-audio-types v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-foundation v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-graphics v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-image v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-media v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-video v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-encode v4.1.0 | MIT |
| objc2-foundation v0.2.2 | MIT |
| objc2-foundation v0.3.2 | MIT |
| objc2-image-io v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-media-toolbox v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-metal v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-quartz-core v0.3.2 | Zlib OR Apache-2.0 OR MIT |
| once_cell v1.21.4 | MIT OR Apache-2.0 |
| ort v2.0.0-rc.13 | MIT OR Apache-2.0 |
| ort-sys v2.0.0-rc.13 | MIT OR Apache-2.0 |
| parking_lot v0.12.5 | MIT OR Apache-2.0 |
| parking_lot_core v0.9.12 | MIT OR Apache-2.0 |
| peniko v0.6.1 | Apache-2.0 OR MIT |
| percent-encoding v2.3.2 | MIT OR Apache-2.0 |
| pin-project-lite v0.2.17 | Apache-2.0 OR MIT |
| png v0.18.1 | MIT OR Apache-2.0 |
| pollster v1.0.1 | Apache-2.0/MIT |
| polycool v0.4.0 | MIT OR Apache-2.0 |
| portable-atomic v1.15.0 | Apache-2.0 OR MIT |
| potential_utf v0.1.6 | Unicode-3.0 |
| primal-check v0.3.4 | MIT OR Apache-2.0 |
| proc-macro2 v1.0.107 | MIT OR Apache-2.0 |
| profiling v1.0.18 | MIT OR Apache-2.0 |
| pxfm v0.1.30 | BSD-3-Clause OR Apache-2.0 |
| quick-error v2.0.1 | MIT/Apache-2.0 |
| quote v1.0.47 | MIT OR Apache-2.0 |
| raw-window-handle v0.6.2 | MIT OR Apache-2.0 OR Zlib |
| raw-window-metal v1.1.0 | MIT OR Apache-2.0 |
| rawpointer v0.2.1 | MIT/Apache-2.0 |
| rayon v1.12.0 | MIT OR Apache-2.0 |
| rayon-core v1.13.0 | MIT OR Apache-2.0 |
| read-fonts v0.41.0 | MIT OR Apache-2.0 |
| realfft v3.5.0 | MIT |
| regex v1.13.1 | MIT OR Apache-2.0 |
| regex-automata v0.4.18 | MIT OR Apache-2.0 |
| regex-syntax v0.8.11 | MIT OR Apache-2.0 |
| rfd v0.17.2 | MIT |
| rubato v0.15.0 | MIT |
| rustc-hash v1.1.0 | Apache-2.0/MIT |
| rustc-hash v2.1.3 | Apache-2.0 OR MIT |
| rustfft v6.4.1 | MIT OR Apache-2.0 |
| rustversion v1.0.23 | MIT OR Apache-2.0 |
| same-file v1.0.6 | Unlicense/MIT |
| scopeguard v1.2.0 | MIT OR Apache-2.0 |
| self_cell v1.3.0 | Apache-2.0 OR GPL-2.0-only |
| seq-macro v0.3.6 | MIT OR Apache-2.0 |
| serde v1.0.229 | MIT OR Apache-2.0 |
| serde_core v1.0.229 | MIT OR Apache-2.0 |
| serde_derive v1.0.229 | MIT OR Apache-2.0 |
| serde_json v1.0.151 | MIT OR Apache-2.0 |
| sha2 v0.10.9 | MIT OR Apache-2.0 |
| simd-adler32 v0.3.10 | MIT |
| skrifa v0.44.0 | MIT OR Apache-2.0 |
| smallvec v1.16.2 | MIT OR Apache-2.0 |
| smol_str v0.2.2 | MIT OR Apache-2.0 |
| stable_deref_trait v1.2.1 | MIT OR Apache-2.0 |
| static_assertions v1.1.0 | MIT OR Apache-2.0 |
| strength_reduce v0.2.4 | MIT OR Apache-2.0 |
| syn v2.0.119 | MIT OR Apache-2.0 |
| syn v3.0.6 | MIT OR Apache-2.0 |
| synstructure v0.14.0 | MIT |
| thiserror v1.0.69 | MIT OR Apache-2.0 |
| thiserror v2.0.21 | MIT OR Apache-2.0 |
| thiserror-impl v1.0.69 | MIT OR Apache-2.0 |
| thiserror-impl v2.0.21 | MIT OR Apache-2.0 |
| tiff v0.11.3 | MIT |
| tinystr v0.8.4 | Unicode-3.0 |
| tracing v0.1.44 | MIT |
| tracing-core v0.1.36 | MIT |
| transpose v0.2.3 | MIT OR Apache-2.0 |
| type-map v0.5.1 | MIT/Apache-2.0 |
| typenum v1.20.1 | MIT OR Apache-2.0 |
| unicode-general-category v1.1.0 | Apache-2.0 |
| unicode-ident v1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| unicode-segmentation v1.13.3 | MIT OR Apache-2.0 |
| unicode-width v0.2.2 | MIT OR Apache-2.0 |
| url v2.5.8 | MIT OR Apache-2.0 |
| utf8_iter v1.0.4 | Apache-2.0 OR MIT |
| utf8parse v0.2.2 | Apache-2.0 OR MIT |
| uuid v1.26.1 | Apache-2.0 OR MIT |
| vello_common v0.1.0 | Apache-2.0 OR MIT |
| vello_cpu v0.1.0 | Apache-2.0 OR MIT |
| walkdir v2.5.0 | Unlicense/MIT |
| web-time v1.1.0 | MIT OR Apache-2.0 |
| webbrowser v1.2.4 | MIT OR Apache-2.0 |
| weezl v0.1.12 | MIT OR Apache-2.0 |
| wgpu v30.0.1 | MIT OR Apache-2.0 |
| wgpu-core v30.0.1 | MIT OR Apache-2.0 |
| wgpu-core-deps-apple v30.0.1 | MIT OR Apache-2.0 |
| wgpu-hal v30.0.1 | MIT OR Apache-2.0 |
| wgpu-naga-bridge v30.0.1 | MIT OR Apache-2.0 |
| wgpu-types v30.0.1 | MIT OR Apache-2.0 |
| winit v0.30.13 | Apache-2.0 |
| writeable v0.6.4 | Unicode-3.0 |
| yoke v0.8.3 | Unicode-3.0 |
| yoke-derive v0.8.3 | Unicode-3.0 |
| zerocopy v0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerocopy-derive v0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerofrom v0.1.8 | Unicode-3.0 |
| zerofrom-derive v0.1.8 | Unicode-3.0 |
| zerotrie v0.2.5 | Unicode-3.0 |
| zerovec v0.11.8 | Unicode-3.0 |
| zerovec-derive v0.11.6 | Unicode-3.0 |
| zmij v1.0.23 | MIT |
| zune-core v0.5.3 | MIT OR Apache-2.0 OR Zlib |
| zune-jpeg v0.5.15 | MIT OR Apache-2.0 OR Zlib |
