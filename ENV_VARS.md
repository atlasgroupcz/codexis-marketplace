# Codexis VM Environment Variables

Source of truth for environment variables that live inside each user VM
(`~/.cdx/.env`) and are read by marketplace plugins. The daemon
(`cdx-daemon`) is the **sole producer** of this file via
`VirtualMachineEnvFile.write(uid)` — plugins are pure consumers.

## Naming convention

- `CODEXIS_PUBLIC_*` — non-secret config (URLs, IDs, paths, session context).
  Safe to log, may appear in plugin diagnostics.
- `CODEXIS_USER_*` — per-user secrets (API tokens, credentials).
  **Never** log, never include in telemetry, never embed in frontend bundles.

Anything outside these two prefixes that lives in `~/.cdx/.env` is
user-managed configuration that the daemon does not touch (it is preserved
across rewrites; only known producer-owned keys are overwritten).

## Public (non-secret) variables

| Variable | Producer | Consumer | Purpose |
|---|---|---|---|
| `CODEXIS_PUBLIC_BASE_URL` | user `.env` | `cdx-link-rewriter`, daemon `OnRenderRunner` | Codexis web base URL |
| `CODEXIS_PUBLIC_API_URL` | user `.env` | `cdx-cli` | Codexis REST API root |
| `CODEXIS_PUBLIC_AT_API_URL` | user `.env` | `cdx-at` | Austrian regional API |
| `CODEXIS_PUBLIC_CZ_PSP_API_URL` | user `.env` | `cdx-cz-psp` | Czech PSP regional API |
| `CODEXIS_PUBLIC_CZ_SPP_API_URL` | user `.env` | `cdx-cz-spp` | Czech SPP regional API |
| `CODEXIS_PUBLIC_SK_API_URL` | user `.env` | `cdx-sk` | Slovak regional API |
| `CODEXIS_PUBLIC_DAEMON_URL` | daemon shell injection | `cdxctl`, `sledovane_dokumenty_core` | cdx-daemon GraphQL endpoint (default `http://localhost:8086/graphql`) |
| `CODEXIS_PUBLIC_SESSION_ID` | daemon shell injection | `video-analyze`, `sledovane_dokumenty_core` | Per-session ID (not secret, but per-session) |
| `CODEXIS_PUBLIC_USER_HOME` | daemon shell injection | `katastr_core` | User home root inside VM |
| `CODEXIS_PUBLIC_LITELLM_BASE_URL` | daemon `LiteLlmCredentialsProvider` | `video-analyze`, daemon chat/transcript resolvers | LiteLLM proxy base URL |

## User-secret variables

| Variable | Producer | Consumer | Purpose |
|---|---|---|---|
| `CODEXIS_USER_API_TOKEN` | daemon `CodexisApiTokenProvider` | all `cdx-*` plugins | Codexis API auth (Bearer token) |
| `CODEXIS_USER_LITELLM_API_KEY` | daemon `LiteLlmCredentialsProvider` | `video-analyze`, daemon chat/transcript resolvers | Per-user LiteLLM API key |
| `CODEXIS_USER_DAEMON_TOKEN` | daemon `DaemonTokenProvider` | (reserved for future plugins calling back to daemon) | 24h JWT for VM → daemon callback |

## Producer/consumer flow

```
docker run cdx-runtime
        │
        ▼
VmEndpointResolvedEvent
        │
        ▼
VirtualMachineEnvFileWriter (Spring listener)
        │
        ▼
VirtualMachineEnvFile.write(uid)
        │
        ├── reads existing ~/.cdx/.env (preserves user-managed keys)
        ├── rewrites legacy keys (LITELLM_API_KEY → CODEXIS_USER_LITELLM_API_KEY, …)
        ├── invokes every VirtualMachineEnvProvider bean:
        │     • CodexisApiTokenProvider     → CODEXIS_USER_API_TOKEN
        │     • LiteLlmCredentialsProvider  → CODEXIS_USER_LITELLM_API_KEY,
        │                                     CODEXIS_PUBLIC_LITELLM_BASE_URL
        │     • DaemonTokenProvider         → CODEXIS_USER_DAEMON_TOKEN
        │
        ▼
~/.cdx/.env (single source of truth)
        │
        ▼
VmShellSandboxEnvLoader reads at every shell invocation
        │
        ▼
plugin process (Rust / Python / bash)
```

## Adding a new env variable

1. Decide prefix: public or user-secret?
2. If daemon should produce it, add a `VirtualMachineEnvProvider` bean in
   `cdx-daemon/cdx-daemon-app/.../vm/service/` (or domain-specific package).
3. Update this registry.
4. Update `cdx-daemon/build/ci/check-legacy-envs.sh` if you are renaming
   away from a legacy name (add the legacy name to the guard pattern).

## Migration of legacy names

Legacy → new mapping is hardcoded in
`VirtualMachineEnvFile.LEGACY_KEY_REWRITES` and applied during every
`write(uid)`. After one provisioning cycle, every active VM has the new
key names. Old names are also rejected by `check-legacy-envs.sh` to
prevent regression.
