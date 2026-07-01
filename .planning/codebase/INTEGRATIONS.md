# External Integrations

**Analysis Date:** 2026-05-06

## APIs & External Services

**Identity and auth providers:**
- GitHub OAuth via `next-auth/providers/github` in `website/website_2025-09-30/src/configs/auth.config.ts`.
  - SDK / client: `next-auth` 5 beta.
  - Auth: `GITHUB_AUTH_CLIENT_ID`, `GITHUB_AUTH_CLIENT_SECRET`.
- Google OAuth via `next-auth/providers/google` in the same config.
  - SDK / client: `next-auth` 5 beta.
  - Auth: `GOOGLE_AUTH_CLIENT_ID`, `GOOGLE_AUTH_CLIENT_SECRET`.
- Credentials sign-in via `next-auth/providers/credentials` with backend validation in `website/website_2025-09-30/src/server/actions/user/validateCredential`.
  - Auth: custom backend credential check rather than a third-party identity service.

**Content and rendering services:**
- Google Fonts Web Fonts API in `website/website_2025-09-30/src/server/actions/fonts.ts`.
  - SDK / client: server-side `fetch` to `https://www.googleapis.com/webfonts/v1/webfonts`.
  - Auth: `GOOGLE_FONTS_API_KEY`.
- Kroki diagram rendering in `website/website_2025-09-30/src/app/(protected-pages)/(docs)/[...slug]/page.tsx`.
  - SDK / client: `@kazumatu981/markdown-it-kroki`.
  - Auth: none.
  - Local dev service: `yuzutech/kroki` in `website/website_2025-09-30/docker-compose.yml`.
  - Operator note: `website/USAGE.md` documents `NEXT_PUBLIC_KROKI_SERVER_URL` for local and hosted rendering.

**Browser automation and test runtime:**
- Headless Firefox WebDriver capabilities in `webdriver.json` and `crates/z00z_wallets/webdriver.json`.
  - SDK / client: WebDriver-compatible browser test tooling.
  - Auth: none.
  - Purpose: browser automation and WASM test execution rather than production integration.

**Internal service boundary:**
- Wallet JSON-RPC via `jsonrpsee` in `crates/z00z_networks/rpc` and `crates/z00z_wallets/src/adapters/rpc/`.
  - SDK / client: `jsonrpsee` 0.26.
  - Role: internal transport and application boundary, not a third-party API.

**Not detected in active source:**
- Payment processors.
- Email delivery providers.
- Chat or notification SaaS providers.
- External webhook receivers or webhook delivery platforms.

## Data Storage

**Databases:**
- RedB embedded wallet store in `crates/z00z_wallets/src/db/redb_wallet_store.rs` and `crates/z00z_wallets/Cargo.toml`.
  - Client: `redb` 3.1.0.
  - Purpose: native `.wlt` persistence for wallet identity, scan state, TOFU pins, stealth metadata, and encrypted records.
- JMT-backed asset storage in `crates/z00z_storage/src/assets/store.rs` and `crates/z00z_storage/src/assets/store_internal/redb_backend.rs`.
  - Client: `jmt` 0.12.0 plus `redb` in the storage backend.
  - Purpose: canonical asset state, proofs, checkpoints, and snapshot artifacts.
- IndexedDB in the browser wallet runtime via `rexie` in `crates/z00z_wallets/src/wasm/indexeddb_backend.rs`.
  - Purpose: browser-side wallet persistence with the same logical KV/blob contracts as native RedB.

**File storage:**
- Local filesystem storage is a first-class integration surface.
  - `crates/z00z_core/src/genesis/genesis.rs` writes genesis archives and generated reports.
  - `crates/z00z_storage/src/snapshot/store.rs` and `crates/z00z_storage/src/checkpoint/store.rs` persist snapshot and checkpoint artifacts.
  - `crates/z00z_simulator/src/scenario_1/*.rs` materialize JSON, markdown, and XLSX outputs.
  - `crates/z00z_wallets/src/db/redb_wallet_store.rs` and `crates/z00z_simulator/src/scenario_1/stage_2.rs` read and write wallet `.wlt` files.

**Caching:**
- In-memory only in the inspected target subsystems.
  - `z00z_storage` keeps tree and root state in memory in `crates/z00z_storage/src/assets/store.rs`.
  - `z00z_wallets` exposes cache facades from `crates/z00z_wallets/src/lib.rs`.
  - No Redis, Memcached, or other external cache service was detected.

## Authentication & Identity

**Auth provider:**
- NextAuth in the website subtree.
  - Implementation: `website/website_2025-09-30/src/configs/auth.config.ts`, `website/website_2025-09-30/src/middleware.ts`, `website/website_2025-09-30/src/_auth.ts`, and `website/website_2025-09-30/src/components/auth/AuthProvider/AuthProvider.tsx`.
  - Providers: GitHub OAuth, Google OAuth, and credentials-based login.
  - Environment-aware behavior: `website/website_2025-09-30/src/middleware.ts` reads `NEXT_PUBLIC_ENVIRONMENT` to alter sign-in behavior in test mode.
- Wallet identity remains internal and cryptographic rather than third-party.
  - Implementation: `crates/z00z_crypto/src/lib.rs`, `crates/z00z_wallets/src/core/key/`, and `crates/z00z_wallets/src/services/seed_phrase.rs`.
  - No external SSO, OIDC, or hosted identity provider was detected in the Rust workspace.

## Monitoring & Observability

**Error tracking:**
- None detected as an external service.

**Logs and metrics:**
- `tracing` is present across the Rust workspace, including `crates/z00z_core`, `crates/z00z_utils`, `crates/z00z_networks/rpc`, and `crates/z00z_wallets`.
- `crates/z00z_utils/src/metrics/traits.rs` defines a `MetricsSink` abstraction, and `crates/z00z_utils/Cargo.toml` exposes an optional `prometheus` feature.
- Wallet RPC logging is wired through `crates/z00z_wallets/src/adapters/rpc/logging/` and consumed by simulator stage flows.
- No external APM or hosted log service such as Sentry, Datadog, or New Relic was detected.

## CI/CD & Deployment

**Hosting:**
- `website/website_2025-09-30/Dockerfile` builds the active Next.js app.
- `website/website_2025-09-30/docker-compose.yml` runs the website together with a local `kroki` container.
- Native Rust execution targets remain local binaries and libraries; no cloud deployment target is declared in the inspected files.
- The root `docker/Dockerfile` is still present, but it targets a legacy `zuz-node` layout that does not match the current workspace tree.

**CI pipeline:**
- Repository-local build orchestration lives in `scripts/cargo_build.py` and `scripts/cargo_build.sh`.
- The build helper reads `versions.yaml` and `scripts/cargo_build_config.yaml` to decide which crates to build.

## Environment Configuration

**Required env vars:**
- `GITHUB_AUTH_CLIENT_ID` and `GITHUB_AUTH_CLIENT_SECRET` - GitHub OAuth in `website/website_2025-09-30/src/configs/auth.config.ts`.
- `GOOGLE_AUTH_CLIENT_ID` and `GOOGLE_AUTH_CLIENT_SECRET` - Google OAuth in the same config.
- `GOOGLE_FONTS_API_KEY` - Google Fonts API access in `website/website_2025-09-30/src/server/actions/fonts.ts`.
- `NEXT_PUBLIC_KROKI_SERVER_URL` - Kroki endpoint override in `website/website_2025-09-30/src/app/(protected-pages)/(docs)/[...slug]/page.tsx`.
- `NEXT_PUBLIC_ENVIRONMENT` - Middleware behavior switch in `website/website_2025-09-30/src/middleware.ts`.
- `Z00Z_BUILD_DEV_ONLY` - Build script gate for dev-only crates in `scripts/cargo_build.py`.
- `Z00Z_WALLET_NETWORK` and `Z00Z_WALLET_CHAIN` - Simulator stage 2 wallet setup in `crates/z00z_simulator/src/scenario_1/stage_2.rs`.

**Secrets location:**
- No plaintext secret store was inspected.
- Runtime secrets are expected to live in environment variables or external secret storage, not in repository files.
- `config/z00z_blockchain_config.yaml` exists but currently contains only whitespace.

## Webhooks & Callbacks

**Incoming:**
- No active webhook endpoint was detected in production source.

**Outgoing:**
- No active webhook delivery integration was detected in production source.
- The only webhook-like references are documentation and reference material, such as `website/USAGE.md` and `crates/z00z_wallets/src/egui_views/ref-docs/tari/tari-EVENT_SYSTEM_ADOPTION_GUIDE.md`.

---

*Integration audit: 2026-05-06*
