# Verification Toolchain Install Audit

**Date:** 2026-06-16  
**Repository:** Z00Z  
**Scope:** `scripts/verification-tools/install-verification-tools.sh`, `scripts/install-verification-tools.log`, current workstation state

## ✅ Conclusions

- `./scripts/verification-tools/install-verification-tools.sh --self-test --profile all --strict` passed with exit code `0`.
- The latest post-fix replay install completed in `44.12s` real time.
- The latest strict self-test completed in `13.79s` real time.
- All non-`sudo` confirmation prompts in `scripts/install-verification-tools.log` were auto-answered with `y`.
- The original top-level install recorded in `scripts/install-verification-tools.log` ran once, not twice.
- The current installer now avoids the previously observed pitfalls:
  - Tamarin no longer relies on unsupported Ubuntu `maude 3.2`; it uses local `Maude 3.5.1`.
  - Tamarin wrapper recursion was fixed by separating wrapper and upstream executable paths.
  - Dudect self-test was changed from flaky upstream `test.py` semantics to a bounded smoke test.
  - Kani setup no longer needslessly re-enters the bundle bootstrap when a ready bundle already exists.

## 🔍 Evidence

### 📌 Top-Level Log Stage Counts

From `scripts/install-verification-tools.log`:

| Stage marker | Count |
|---|---:|
| `Installing system packages with apt-get` | 1 |
| `Updating Rust toolchains` | 1 |
| `Installing ProVerif, Why3, and Alt-Ergo` | 1 |
| `Installing EasyCrypt` | 1 |
| `Installing hax from source checkout` | 1 |

Interpretation: the install did not run twice end-to-end. The repeated feeling came from nested installers such as OPAM, hax setup, and Kani bundle setup.

### 🔔 Prompt Handling

Observed prompt lines in `scripts/install-verification-tools.log`:

- Manual prompt:
  - line `3`: `[sudo] password for vadim:`
- Auto-answered prompts:
  - line `3771`: `(answer 'n' for other options) [Y/n] y`
  - line `3853`: `Do you want to continue? [Y/n]`
  - line `4359`: `Package easycrypt does not exist, create as a NEW package? [Y/n] y`
  - line `4545`: `Package creusot-deps does not exist, create as a NEW package? [Y/n] y`
  - line `4553`: `Pin and install them? [Y/n] y`
  - line `4621`: `(answer 'n' for other options) [Y/n] y`
  - line `4677`: `Do you want to continue? [Y/n]`
  - line `5462`: `Package hax-engine does not exist, create as a NEW package? [Y/n] y`

### ⚙️ Verification Commands

Commands that were re-run successfully after the fixes:

```bash
./scripts/verification-tools/install-verification-tools.sh --install --profile recommended --skip-system --skip-node --skip-opam
./scripts/verification-tools/install-verification-tools.sh --self-test --profile all --strict
```

## 📦 Installed Inventory

### ⚙️ Rust and Cargo Binaries

| Tool | Path |
|---|---|
| `rustup` | `/home/vadim/.cargo/bin/rustup` |
| `cargo` | `/home/vadim/.cargo/bin/cargo` |
| `rustc` | `/home/vadim/.cargo/bin/rustc` |
| `cargo-nextest` | `/home/vadim/.cargo/bin/cargo-nextest` |
| `cargo-audit` | `/home/vadim/.cargo/bin/cargo-audit` |
| `cargo-deny` | `/home/vadim/.cargo/bin/cargo-deny` |
| `cargo-vet` | `/home/vadim/.cargo/bin/cargo-vet` |
| `cargo-fuzz` | `/home/vadim/.cargo/bin/cargo-fuzz` |
| `cargo-geiger` | `/home/vadim/.cargo/bin/cargo-geiger` |
| `cargo-kani` | `/home/vadim/.cargo/bin/cargo-kani` |
| `cargo-llvm-cov` | `/home/vadim/.cargo/bin/cargo-llvm-cov` |
| `cargo-semver-checks` | `/home/vadim/.cargo/bin/cargo-semver-checks` |
| `just` | `/home/vadim/.cargo/bin/just` |
| `bacon` | `/home/vadim/.cargo/bin/bacon` |
| `watchexec` | `/home/vadim/.cargo/bin/watchexec` |
| `mdbook` | `/home/vadim/.cargo/bin/mdbook` |
| `lychee` | `/home/vadim/.cargo/bin/lychee` |
| `taplo` | `/home/vadim/.cargo/bin/taplo` |

Selected script-installed Cargo binaries occupy `211.5 MB` in total.

### 🔑 System-Managed Binaries

| Tool | Path |
|---|---|
| `java` | `/usr/bin/java` |
| `jq` | `/usr/bin/jq` |
| `shellcheck` | `/usr/bin/shellcheck` |
| `opam` | `/usr/bin/opam` |
| `z3` | `/usr/bin/z3` |
| `node` | `/home/vadim/.nvm/versions/node/v22.20.0/bin/node` |
| `npm` | `/home/vadim/.nvm/versions/node/v22.20.0/bin/npm` |
| `system maude` | `/usr/bin/maude` |

### 🧪 Repository-Local Formal Tools

| Tool | Path |
|---|---|
| `local Maude wrapper` | `/home/vadim/Projects/z00z/tools/formal_verification/maude/bin/maude` |
| `Tamarin wrapper` | `/home/vadim/Projects/z00z/tools/formal_verification/tamarin/bin/tamarin-prover-z00z` |
| `Tamarin compatibility symlink` | `/home/vadim/Projects/z00z/tools/formal_verification/tamarin/bin/tamarin-prover` |
| `Tamarin upstream executable` | `/home/vadim/Projects/z00z/tools/formal_verification/tamarin/upstream/tamarin-prover` |
| `TLA+` | `/home/vadim/Projects/z00z/tools/formal_verification/tla/tla2tools.jar` |
| `Alloy` | `/home/vadim/Projects/z00z/tools/formal_verification/alloy/org.alloytools.alloy.dist.jar` |
| `Apalache` | `/home/vadim/Projects/z00z/tools/formal_verification/apalache/bin/apalache-mc` |
| `Verus` | `/home/vadim/Projects/z00z/tools/formal_verification/verus/bin/verus` |
| `cargo-prusti` | `/home/vadim/Projects/z00z/tools/formal_verification/prusti/bin/cargo-prusti` |
| `prusti-rustc` | `/home/vadim/Projects/z00z/tools/formal_verification/prusti/bin/prusti-rustc` |
| `dudect` | `/home/vadim/Projects/z00z/tools/formal_verification/dudect` |
| `hax` | `/home/vadim/Projects/z00z/tools/formal_verification/hax` |

Per-directory sizes:

```text
1.3M  tools/formal_verification/dudect
2.2M  tools/formal_verification/tla
9.4M  tools/formal_verification/maude
21M   tools/formal_verification/alloy
63M   tools/formal_verification/tamarin
144M  tools/formal_verification/verus
379M  tools/formal_verification/apalache
461M  tools/formal_verification/prusti
677M  tools/formal_verification/creusot
1.6G  tools/formal_verification/hax
```

### 📦 npm Global Packages

Global npm root:

```text
/home/vadim/.nvm/versions/node/v22.20.0/lib/node_modules
```

Installed global packages currently visible there:

```text
@anthropic-ai/claude-code 2.1.87
@github/copilot 1.0.11
@gsd-build/sdk null
@modelcontextprotocol/server-gdrive 2025.1.14
@openai/codex 0.139.0
corepack 0.34.0
gsd-pi 2.80.0
lat.md 0.11.0
markdownlint-cli2 0.22.1
npm 11.14.1
promptfoo 0.121.13
```

The package directly installed by this script is:

```text
/home/vadim/.nvm/versions/node/v22.20.0/lib/node_modules/markdownlint-cli2
```

Its measured size is `15M`.

### 🔑 OPAM Switch

Switch root:

```text
/home/vadim/.opam/z00z-verify
```

Installed package count:

```text
152
```

Installed OPAM packages:

```text
alt-ergo
alt-ergo-lib
alt-ergo-parsers
angstrom
astring
base
base-bigarray
base-domains
base-nnp
base-threads
base-unix
base_bigstring
base_quickcheck
batteries
bigstringaf
bin_prot
camlp-streams
camlzip
capitalization
cmdliner
conf-gmp
conf-gtk2
conf-libpcre2-8
conf-pkg-config
conf-zlib
core
cppo
csexp
cstruct
dolmen
dolmen_loop
dolmen_type
dune
dune-build-info
dune-compiledb
dune-configurator
dune-private-libs
dune-site
dyn
easycrypt
ezjsonm
fieldslib
fmt
fpath
fs-io
gel
gen
hax-engine
hex
hmap
int_repr
jane-street-headers
js_of_ocaml
js_of_ocaml-compiler
js_of_ocaml-ppx
jsonm
jst-config
lablgtk
logs
markdown
menhir
menhirCST
menhirLib
menhirSdk
non_empty_list
num
ocaml
ocaml-base-compiler
ocaml-compiler-libs
ocaml-config
ocaml-options-vanilla
ocaml-syntax-shims
ocaml_intrinsics_kernel
ocamlbuild
ocamlfind
ocamlgraph
ocplib-simplex
octavius
ordering
parsexp
pcre2
pp
pp_loc
pprint
ppx_assert
ppx_base
ppx_bench
ppx_bin_prot
ppx_blob
ppx_cold
ppx_compare
ppx_custom_printf
ppx_derivers
ppx_deriving
ppx_deriving_yojson
ppx_diff
ppx_disable_unused_warnings
ppx_enumerate
ppx_expect
ppx_fields_conv
ppx_fixed_literal
ppx_globalize
ppx_hash
ppx_here
ppx_ignore_instrumentation
ppx_inline_test
ppx_jane
ppx_js_style
ppx_let
ppx_log
ppx_matches
ppx_module_timer
ppx_optcomp
ppx_optional
ppx_pipebang
ppx_sexp_conv
ppx_sexp_message
ppx_sexp_value
ppx_stable
ppx_stable_witness
ppx_string
ppx_string_conv
ppx_tydi
ppx_typerep_conv
ppx_variants_conv
ppx_yojson_conv
ppx_yojson_conv_lib
ppxlib
ppxlib_jane
proverif
psmt2-frontend
re
sedlex
seq
sexplib
sexplib0
spelll
splittable_random
stdint
stdio
stdlib-shims
stdune
time_now
top-closure
topkg
typerep
tyxml
uutf
variantslib
why3
yojson
zarith
```

### 📦 APT Packages Installed During the Recorded Bootstrap

These were extracted from `/var/log/apt/history.log`.

#### ⚙️ Core Bootstrap (`2026-06-16 00:19:42`)

```text
opam
mercurial-common
libamd3
libice-dev
openjdk-17-jdk
openjdk-17-jre
openjdk-17-jdk-headless
ocaml-interp
shellcheck
libsm-dev
opam-installer
mercurial
libstdlib-ocaml
ocaml-base
ocaml
libcompiler-libs-ocaml-dev
ocaml-man
libstdlib-ocaml-dev
ledit
libxt-dev
z3
openjdk-17-jre-headless
libglpk40
```

Installed-size total: `768.8 MB`

#### 📚 OPAM External Deps (`2026-06-16 00:29:48`)

```text
libblkid-dev
debhelper
libselinux1-dev
libglib2.0-dev-bin
libxcomposite-dev
dwz
libmount-dev
libxrender-dev
libglib2.0-dev
libxml2-utils
libbz2-dev
libjpeg-dev
libgtk2.0-dev
libwebpdecoder3
libwebp-dev
libgmpxx4ldbl
libxrandr-dev
libpng-dev
libxfixes-dev
libfribidi-dev
liblerc-dev
libtiffxx6
libsub-override-perl
po-debconf
debugedit
libdebhelper-perl
libxinerama-dev
gir1.2-gtk-2.0
libjbig-dev
libfontconfig-dev
libxi-dev
libpcre2-posix3
libxcb-shm0-dev
gir1.2-freedesktop-dev
libgirepository-2.0-0
autopoint
gir1.2-glib-2.0-dev
libdatrie-dev
libharfbuzz-cairo0
libatk1.0-dev
libgdk-pixbuf-2.0-dev
libdeflate-dev
libxft-dev
libjpeg-turbo8-dev
pango1.0-tools
libxcursor-dev
libsepol-dev
uuid-dev
libxcb-render0-dev
libxdamage-dev
libbrotli-dev
libsys-hostname-long-perl
libgraphite2-dev
dh-autoreconf
libmail-sendmail-perl
libpng-tools
libxext-dev
libfreetype-dev
dh-strip-nondeterminism
libfile-stripnondeterminism-perl
libcairo2-dev
libgmp-dev
libtiff-dev
libpcre2-dev
libpixman-1-dev
libjpeg8-dev
libharfbuzz-dev
libsharpyuv-dev
libpango1.0-dev
libarchive-cpio-perl
libthai-dev
bzip2-doc
```

Installed-size total: `64.6 MB`

#### 🧩 Creusot GTK Deps (`2026-06-16 00:34:05`)

```text
libegl-dev
libatk-bridge2.0-dev
libgles-dev
libgtksourceview-3.0-dev
libxtst-dev
libatspi2.0-dev
libepoxy-dev
libglvnd-dev
libgles1
libegl1-mesa-dev
wayland-protocols
libglvnd-core-dev
libxkbcommon-dev
libdbus-1-dev
libwayland-bin
libgtk-3-dev
libwayland-dev
libopengl-dev
```

Installed-size total: `22.7 MB`

#### 🔔 Manual Follow-Up (`2026-06-16 01:09:42`)

This was the user-assisted `sudo apt-get install -y maude` step, not part of the final installer logic after the fixes:

```text
maude
libsigsegv2
libtecla1t64
libbdd0c2
```

Installed-size total: `3.2 MB`

## 💽 Disk Footprint

### 📏 Measured User-Space Footprint

```text
3.3G  /home/vadim/Projects/z00z/tools/formal_verification
4.1G  /home/vadim/.opam/z00z-verify
```

Additional script-owned measured subsets:

- selected Cargo binaries: `211.5 MB`
- `markdownlint-cli2`: `15 MB`

Measured user-space subtotal: approximately `7.62 GiB`

### 📐 APT Installed-Size Totals

- core bootstrap: `768.8 MB`
- OPAM external deps: `64.6 MB`
- Creusot GTK deps: `22.7 MB`

APT subtotal attributable to the bootstrap history: approximately `0.84 GiB`

### ✅ Combined Estimate

Mixed-method combined estimate for the verified toolchain footprint:

```text
~8.46 GiB
```

Notes:

- This combines direct filesystem measurements for user-space tools with `dpkg` installed-size metadata for APT packages.
- It excludes unrelated pre-existing global npm packages and unrelated existing Cargo binaries.
- It treats the current local `Maude 3.5.1` as part of `tools/formal_verification`.

## ⏱️ Timing

- latest replay install after fixes: `44.12s` real, `4.17s` user, `1.33s` sys
- latest strict self-test: `13.79s` real, `4.52s` user, `9.68s` sys

What cannot be reconstructed precisely:

- the wall-clock duration of the original first install from `scripts/install-verification-tools.log`

Reason:

- `scripts/install-verification-tools.log` does not contain full per-line timestamps, so exact elapsed time for the original run is not derivable after the fact.

## 🧹 Uninstall

The script help now includes uninstall guidance:

```bash
./scripts/verification-tools/install-verification-tools.sh --help
```

Canonical removal commands:

```bash
cargo uninstall cargo-vet cargo-semver-checks just bacon watchexec-cli mdbook lychee taplo-cli cargo-geiger kani-verifier
opam switch remove z00z-verify
npm uninstall --global markdownlint-cli2
gio trash ./tools/formal_verification
sudo apt remove opam z3 shellcheck openjdk-17-jdk openjdk-17-jdk-headless openjdk-17-jre openjdk-17-jre-headless ocaml ocaml-base ocaml-interp ocaml-man opam-installer mercurial mercurial-common ledit libamd3 libcompiler-libs-ocaml-dev libglpk40 libice-dev libsm-dev libstdlib-ocaml libstdlib-ocaml-dev libxt-dev
sudo apt autoremove
```
