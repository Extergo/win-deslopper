# Testing strategy

The project uses fast Rust unit tests for domain rules and state transitions, practical UI-logic tests, and screenshot/visual review for substantial visual changes. Future real operations additionally require Windows integration tests across supported builds, editions, architectures, and regions; virtual-machine and rollback tests; elevation-boundary tests; plugin isolation; update-signature verification; offline and corrupted-state cases; failed-operation recovery; and anti-cheat/gaming compatibility where relevant.

For `ui-preview`, tests cover catalogue searching, category filtering, adding and removing planned IDs, planned-count calculation, and enum-to-display mapping implemented in Rust. Test observable behaviour, not incidental private structure. Formatting, check, warning-denied Clippy, tests, application launch, resizing, keyboard focus, and visual inspection form the release gate.

