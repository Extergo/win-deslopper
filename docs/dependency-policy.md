# Dependency policy

Dependencies are added only for a current approved requirement, never speculation. Every direct dependency proposal must record purpose; why the standard library/current dependencies are insufficient; maintenance status; licence; Windows support; security and supply-chain implications; binary-size impact; external-command or network behaviour; `unsafe` use; alternatives; and removal strategy. Transitive capabilities count.

Review `Cargo.toml` and lockfile changes explicitly. Prefer the standard library and existing Slint dependency. Security-sensitive, unmaintained, unclear-licence, command-executing, network-capable, or broad platform crates require security and architecture review. The `ui-preview` milestone adds no dependency unless essential and approved.

