# Development workflow

Use this lifecycle for every task:

`Understand task → Read governance → Inspect existing code → Define scope → Identify risks → Implement smallest coherent change → Add or update tests → Update documentation → Run quality gates → Review diff → Report results`

Avoid unrelated refactors during feature work. Obtain separate explicit milestone approval before implementing real Windows mutations, privilege escalation, plugin execution, remote update execution, authentication, payments, telemetry, backend commands, persistent storage, new network dependencies, package-signing changes, lower compatibility guarantees, or weaker rollback.

Reports list all files and commands, distinguish automated from procedural enforcement, disclose failed/skipped checks, and never claim visual or platform validation that did not occur.

After `bun install --frozen-lockfile`, frontend work runs `bun run check`, `bun run lint`, `bun run test`, and `bun run build`. Desktop integration work also runs `bun run tauri build --debug --no-bundle` and launches `bun run tauri dev` or the built executable for an interactive Windows review. These supplement rather than replace the Rust gates.

The development watchers exclude generated Cargo, SvelteKit, dependency, schema, and log output through `.taurignore`; Vite separately excludes Cargo's `target` directory. Keep generated paths out of both watcher graphs so Windows file locks cannot create rebuild loops or crash the development server.

