# Dependency policy

Dependencies are added only for a current approved requirement, never speculation. Every direct dependency proposal must record purpose; why the standard library/current dependencies are insufficient; maintenance status; licence; Windows support; security and supply-chain implications; binary-size impact; external-command or network behaviour; `unsafe` use; alternatives; and removal strategy. Transitive capabilities count.

Review `Cargo.toml`, `Cargo.lock`, `package.json`, and `package-lock.json` changes explicitly. Prefer the standard library and the dependencies already accepted for the current Tauri/SvelteKit architecture. Security-sensitive, unmaintained, unclear-licence, command-executing, network-capable, or broad platform crates and packages require security and architecture review. The accepted migration inventory and removal strategy are recorded in `docs/dependency-review-tauri-sveltekit.md`; additions beyond it require a new review.

