# Development workflow

Use this lifecycle for every task:

`Understand task → Read governance → Inspect existing code → Define scope → Identify risks → Implement smallest coherent change → Add or update tests → Update documentation → Run quality gates → Review diff → Report results`

Avoid unrelated refactors during feature work. Obtain separate explicit milestone approval before implementing real Windows mutations, privilege escalation, plugin execution, remote update execution, authentication, payments, telemetry, backend commands, persistent storage, new network dependencies, package-signing changes, lower compatibility guarantees, or weaker rollback.

Reports list all files and commands, distinguish automated from procedural enforcement, disclose failed/skipped checks, and never claim visual or platform validation that did not occur.

After `npm ci`, frontend work runs `npm run check`, `npm run lint`, `npm test`, and `npm run build`. Desktop integration work also runs `npm run tauri -- build --debug --no-bundle` and launches `npm run tauri -- dev` or the built executable for an interactive Windows review. These supplement rather than replace the Rust gates.

