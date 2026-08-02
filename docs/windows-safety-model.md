# Windows safety model

Every future real operation follows this lifecycle without skipped stages:

`Inspect → Determine compatibility → Create a change plan → Explain impact → Capture original state → Request explicit user approval → Apply through a supported mechanism → Verify the result → Persist an operation receipt → Offer rollback → Verify rollback`

Optional Windows features, Windows capabilities, services, AppX/MSIX packages, classic applications, registry-backed settings, policies, scheduled tasks, startup entries, shell integrations, and drivers/devices are distinct mechanisms. They must not be represented as one generic module operation.

Deslopper forbids deleting files as uninstall, deleting registry keys without prior-state capture, service changes sourced only from internet lists, wildcard AppX removal without dependency analysis, assuming one Windows build represents all builds, silent application, claiming removal when only a surface was hidden, calling undocumented hacks supported, or bypassing anti-cheat, security controls, licensing, or Windows integrity protections.

In the read-only beta, inspection, applicability, desired-state selection, history, drift, and preview planning are implemented. The lifecycle stops before original-state capture and approval because no executor exists. The legacy three-item catalogue remains mock planning data; the 20-component observation views are live read-only evidence. No current lifecycle stage mutates Windows.

The internal mutation alpha is the sole exception to the final sentence above.
It implements the full lifecycle for three closed, unelevated taskbar
presentation operations only. Normal builds and all other component previews
still stop before execution. Cancellation is allowed before mutation; an atomic
write, verification, or rollback is never blindly interrupted. Automatic drift
repair remains prohibited.
