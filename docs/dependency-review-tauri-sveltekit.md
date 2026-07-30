# Tauri and SvelteKit dependency review

This record approves the direct dependency set and frontend toolchain introduced by D-006 for the `ui-preview` milestone. Versions are pinned by `Cargo.lock` and `bun.lock`. Registry metadata and licences were reviewed on 2026-07-27.

## Why dependencies are required

The Rust standard library cannot host a Windows webview, generate Tauri application resources, serialize typed IPC payloads, or build and validate a Svelte UI. The previous Slint dependencies cannot satisfy the explicitly approved Tauri and SvelteKit migration. Dependencies are limited to the native shell, serialization, static presentation, and the formatting/type/lint/test tools needed to enforce the repository gates.

No Tauri plugin is installed. Production code does not gain filesystem, shell, updater, HTTP, persistence, authentication, payment, telemetry, elevation, or Windows-operation capabilities.

## Frontend toolchain

| Tool | Scope and purpose | Maintenance and licence | Windows, security, size, and external behaviour | Alternatives and removal |
| --- | --- | --- | --- | --- |
| Bun 1.3.14 | Developer prerequisite, package manager, JavaScript runtime, and package-script runner. It creates the reproducible `bun.lock`, installs the npm packages required by SvelteKit, and runs the existing build and quality scripts. Bun is not an application dependency and is not shipped with Deslopper. | Current stable official release; MIT. | Supports Windows 10 version 1809 and newer. The single executable and its global cache remain outside the application bundle. `bun install` contacts the npm registry and writes `node_modules`; Bun restricts arbitrary dependency lifecycle scripts by default. The Tauri build commands invoke only repository-defined Bun scripts and Cargo. Development watchers exclude generated output and Cargo artifacts to avoid Windows file-lock rebuild loops. | npm was replaced to match the maintainer's standard toolchain. Reverting requires restoring the npm prerequisite, command documentation, and `package-lock.json`, then removing `bun.lock`. |

## Rust dependencies

| Dependency | Scope and purpose | Maintenance and licence | Windows, security, size, and external behaviour | Alternatives and removal |
| --- | --- | --- | --- | --- |
| `serde` 1.0.229 | Runtime derive support for the two typed Tauri IPC request/response shapes; hand-written JSON parsing would be less safe and duplicate Tauri's transport contract. | Current, widely maintained; MIT OR Apache-2.0. | Cross-platform and small relative to the shell. It performs no network or command execution. Derive macros are build-time code generation. | Remove with typed IPC or replace only with a reviewed serialization format supported by Tauri. |
| `tauri` 2.11.5 | Runtime native window, embedded static-asset host, managed state, and explicit local commands. `default-features` is disabled; only `wry`, asset compression, and Windows common controls are enabled. | Current Tauri 2 release; MIT OR Apache-2.0. | First-class Windows/WebView2 support. It materially increases the Rust graph but replaces Slint. Tauri, Wry, WebView2 bindings, and platform transitive crates contain reviewed native and `unsafe` implementation code; repository source still forbids `unsafe`. No shell or network plugin is present. Runtime assets are local and unused core commands are stripped. | Keeping Slint was rejected by the approved migration. Raw WebView2 bindings would expand unsafe/platform code. Remove if the native shell is replaced through a new architecture decision. |
| `tauri-build` 2.6.3 | Build-time Tauri configuration, capability schema, Windows resources, and embedded-asset generation; the standard Cargo build script cannot reproduce these safely. | Current matching Tauri build line; MIT OR Apache-2.0. | Runs only during Cargo builds and reads repository configuration/assets. It invokes platform resource tooling as part of compilation, does not add a production network client, and contributes no standalone runtime feature. | Remove with Tauri or replace only with an approved equivalent build integration. |

## JavaScript application dependency

| Dependency | Scope and purpose | Maintenance and licence | Windows, security, size, and external behaviour | Alternatives and removal |
| --- | --- | --- | --- | --- |
| `@tauri-apps/api` 2.11.1 | Production frontend access to `invoke`; avoids private globals and hand-written access to Tauri internals. Only the core invoke entry point is imported. | Current official Tauri package; Apache-2.0 OR MIT. | Browser-compatible TypeScript bundled into static assets. It communicates only with the local Tauri IPC transport. It does not itself grant permissions, perform external network requests, or execute commands. | Remove with Tauri. Global Tauri injection was rejected because it broadens frontend exposure. |

## JavaScript build and quality dependencies

These packages are development-only and are not shipped as Bun processes in the application. `bun install` contacts the npm registry; Vite's development server binds to loopback; the Tauri CLI runs the explicitly configured Bun and Cargo build commands. None creates a production network or shell capability.

| Dependency | Purpose and why current tools are insufficient | Maintenance and licence | Windows and supply-chain/size impact | Removal strategy |
| --- | --- | --- | --- | --- |
| `@tauri-apps/cli` 2.11.4 | Official Tauri development, icon, and build orchestration. | Current official release; Apache-2.0 OR MIT. | Supports Windows; build-time executable package, not bundled into the app. It launches only configured local build commands. | Remove with Tauri. |
| `@sveltejs/adapter-static` 3.0.10 | Emits local SPA files because Tauri cannot host an SSR server. | Current official Svelte package; MIT. | Platform-neutral build tool with no runtime server. Output is embedded by Tauri. | Remove with SvelteKit or if a reviewed static adapter replaces it. |
| `@sveltejs/kit` 2.70.1 | Routing, application structure, generated types, and static build integration required by the approved SvelteKit choice. | Current official release; MIT. | Platform-neutral JavaScript build dependency executed by Bun. Server output is not shipped; only static SPA output is embedded. | Remove through a presentation architecture change. Plain Svelte/Vite did not satisfy the requested SvelteKit migration. |
| `@sveltejs/vite-plugin-svelte` 7.2.0 | Compiles and validates `.svelte` files in Vite. | Current official release; MIT. | Build-time only; native optional transitive packages are lockfile-pinned. | Remove with Svelte or Vite. |
| `svelte` 5.56.8 | Declarative, accessible presentation requested by D-006; hand-written DOM code would be harder to validate and maintain. | Current official release; MIT. | Compiles to a small browser bundle and performs no external I/O. | Remove through a presentation architecture change. |
| `vite` 8.1.5 | Bundles the SvelteKit client and provides the loopback-only development server. | Current actively maintained release; MIT. | Bun-executed build/dev tool. Its optional native transformers are lockfile-pinned; no Vite server ships in production. | Remove with SvelteKit's build integration. |
| `typescript` 6.0.3 | Strict checking for IPC contracts and UI logic. Version 6 is selected because it is within every tool's reviewed peer range; TypeScript 7 is not yet accepted by `typescript-eslint`. | Current compatible release line; Apache-2.0. | Cross-platform compiler, build-time only. | Remove only if the frontend leaves TypeScript and equivalent type safety is approved. |
| `svelte-check` 4.7.4 | Svelte template, accessibility, and TypeScript diagnostics not covered by `tsc` alone. | Current official ecosystem release; MIT. | Cross-platform build-time checker. | Remove with Svelte or replace with an equally strict Svelte-aware checker. |
| `eslint` 10.8.0 | JavaScript/TypeScript/Svelte static analysis beyond type checking. | Current actively maintained release; MIT. | Cross-platform development tool with no production footprint. | Remove only with a reviewed lint replacement. |
| `@eslint/js` 10.0.1 | Official ESLint recommended JavaScript rules for flat configuration. | Current official release; MIT. | Configuration package only; no production footprint. | Remove with ESLint. |
| `eslint-plugin-svelte` 3.22.0 | Svelte-specific correctness and accessibility rules, including keyed list enforcement. | Current maintained release; MIT. | Development-only parser/rules. | Remove with Svelte or replace with an equivalent maintained plugin. |
| `eslint-config-prettier` 10.1.8 | Prevents stylistic lint rules from conflicting with the mandatory formatter. | Current maintained release; MIT. | Configuration only; no production footprint. | Remove if ESLint or Prettier is removed. |
| `globals` 17.8.0 | Audited browser and Node global definitions for ESLint's flat configuration. | Current maintained release; MIT. | Static metadata only; no runtime footprint. | Inline the small required set if dependency reduction becomes a priority. |
| `prettier` 3.9.6 | Deterministic Svelte, TypeScript, CSS, JSON, and configuration formatting. | Current actively maintained release; MIT. | Cross-platform development tool; no production footprint. | Remove only with a reviewed deterministic formatter. |
| `prettier-plugin-svelte` 4.1.1 | Svelte-aware formatting not supplied by Prettier core. | Current maintained release; MIT. | Development-only formatter plugin. | Remove with Svelte or Prettier. |
| `typescript-eslint` 8.65.0 | TypeScript parser and recommended ESLint rules; ESLint core cannot parse typed source. | Current maintained release; MIT. | Development-only parser/rules; TypeScript is held to its supported peer range. | Remove with TypeScript or ESLint. |
| `vitest` 4.1.10 | Fast unit tests for the TypeScript IPC boundary and error behavior using the existing Vite graph. | Current actively maintained release; MIT. | Development-only test runner. It does not require a browser or network for current tests. | Remove only with a reviewed frontend test runner and migrated tests. |

## Security override and lockfile review

`@sveltejs/kit` declares `cookie ^0.6.0`, which initially produced advisory GHSA-pxg6-pf52-xh8x even though Deslopper does not use SvelteKit server cookies. `package.json` overrides that transitive package to fixed version 0.7.2 (MIT). The static build, checks, and tests pass with the override, and `bun audit` reports zero known vulnerabilities.

`Cargo.lock` removes the Slint graph and adds the Tauri/Wry/WebView2 graph. `bun.lock` locks all frontend and development transitives with registry integrity hashes. Tauri has broad transitive platform code and native bindings; this is accepted only for the D-006 window/IPC requirement, not as approval for Tauri plugins or new runtime capabilities.

## Binary-size and rollback assessment

The Tauri executable uses the system WebView2 runtime rather than bundling a browser engine. Asset compression is enabled. JavaScript development tools do not ship in the executable. The migration still materially changes build time, dependency count, and binary composition compared with Slint.

Rollback is a source-level revert of D-006, `Cargo.toml`/`Cargo.lock`, the frontend/configuration files, and the associated documentation. There is no user-data migration because `ui-preview` has no persistence. Dependency removal must also remove obsolete lockfile entries and re-run both Rust and frontend gates.
