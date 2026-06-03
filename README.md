# openchaos

**Status:** This project is under active private development. The public repo is a placeholder for now — we plan to open source it soon with working code, docs, and releases.

---

## What is this?

**OpenChaos** is a polyglot developer tool monorepo — inspired by how [OpenCode](https://github.com/sst/opencode) structures its product, but built for a **Python + Zig** stack instead of a single TypeScript/Bun runtime.

The idea is simple: **one product core**, **one API contract**, and **many thin clients** (CLI, web UI, SDKs) that all talk to the same local or remote API. Performance-critical work lives in Zig; orchestration, plugins, agents, and the HTTP server live in Python. TypeScript stays in the UI layer only.

You get the ergonomics of a modern agent-style dev tool without cramming everything into one language or duplicating business logic across repos.

---

## Architecture (target)

```
contract/          ← OpenAPI (source of truth)
    ↓
sdks/            ← Rust, TypeScript, Go (generated clients)
    ↑
core/python      ← serve, config, plugins, agents
    ↑
bindings/        ← narrow C ABI (Python ↔ Zig only)
    ↑
core/zig         ← hot path: parse, crypto, engine
```

| Layer | Role |
|-------|------|
| **contract/** | API shape — health checks, streaming, tool calls, etc. |
| **core/python/** | Product runtime: orchestration, plugins, local HTTP server |
| **core/zig/** | Native hot path — parsing, crypto, engine loops |
| **sdks/** | Thin generated clients; no product logic |
| **apps/** | Surfaces users touch: CLI, web, docs (desktop later) |
| **infra/** | Deploy staging/production when we go hosted |

**Rules we’re designing around:**

- Business rules live in **one** place (`core/python` *or* `core/zig`), not both.
- Cross-language integration is **HTTP + OpenAPI**; Zig FFI is Python-only.
- SDKs are transport + types + ergonomics — never duplicate core logic.
- Desktop (Tauri, etc.) wraps the same web/API story — no forked UI logic.

---

## What needs to be built

This is early. The repo today is intentionally minimal while the real work happens in private. Here’s the roadmap shape:

### Phase 1 — prove the loop

- [ ] `contract/openapi/` — health + at least one real endpoint
- [ ] `core/python` — local API server (e.g. FastAPI/Starlette on port `4096`)
- [ ] `sdks/typescript` + `apps/web` — generated client, minimal UI on port `4444`
- [ ] `sdks/rust` + `sdks/go` — generated clients + one integration test each
- [ ] `core/zig` — one native function exposed to Python via a narrow C ABI
- [ ] `just dev` (or equivalent) — API + web in one command

### Phase 2 — product surfaces

- [ ] `apps/cli` — user entry (Python or thin wrapper around a Zig binary)
- [ ] `apps/docs` — documentation site
- [ ] SDK codegen in CI (`just gen` / `scripts/gen-sdk.sh`)
- [ ] Lockstep versioning (`VERSION` + release script)

### Phase 3 — ship & scale

- [ ] Release binaries (Zig cross-compile per platform)
- [ ] `infra/` for staging/production
- [ ] Optional `apps/api` for hosted API (Go **or** Rust — pick one)
- [ ] `apps/desktop` when web + local serve are solid
- [ ] `apps/console` only if we need billing/auth SaaS

---

## Why “chaos”?

The name is deliberate: multiple languages, one contract, strict dependency direction — it’s structured chaos. The goal is to move fast on features (Python, TS) without giving up native performance (Zig), while keeping every client honest to the same API.

---

## Contributing & license

We’re not accepting contributions yet — the codebase isn’t public in a meaningful way. Watch this repo; we’ll update the README and add `CONTRIBUTING.md` when we open source.

Licensed under [MIT](./LICENSE).
