# Dependency quality gates

The MSRV workflow checks the declared Rust 1.94.0 with default and all library
features, including all package targets, against the pinned Glory checkout.
The dependency job audits all three lockfiles with cargo-audit 0.22.2 against
one recorded RustSec database revision. It runs on pull requests, main pushes,
manual dispatch and the weekly schedule.

Known vulnerabilities or an incomplete scan fail the gate. Informational
warnings remain visible in the Actions summary and JSON artifacts. The gate
rejects configured advisory ignores, severity cutoffs and platform filters;
it does not hide dependencies that are unreachable on the current runner.
The uploaded `sapid-dependency-audit` artifact contains each lockfile digest,
database revision and complete results, including failures.

The 2026-10-30 scan found no known vulnerabilities and four maintenance notices
in the Native lockfile:

| Advisory | Package | Dependency path and follow-up |
|---|---|---|
| RUSTSEC-2026-0192 | ttf-parser 0.25.1 | owned_ttf_parser / ab_glyph / sctk-adwaita / wingy-wayland; coordinate a maintained font-parser migration in the native window engine |
| RUSTSEC-2026-0371 | zbus_names 4.3.5 | AT-SPI / zbus / accesskit_unix / apace_accesskit; coordinate the upstream D-Bus API migration |
| RUSTSEC-2026-0370 | zvariant 5.15.1 | The same Unix accessibility stack; migrate together with its consumers |
| RUSTSEC-2026-0372 | zvariant_derive 5.15.1 | The same Unix accessibility stack; migrate together with zvariant |

These packages were absent from the Windows target's reverse dependency graph,
but remain in the cross-platform lockfile and all reports. These are maintenance
notices, not demonstrated vulnerabilities or an advisory allowlist. Their
resolution requires changes to native engine/accessibility dependencies; do not
replace the pinned engine revision with an unverified one to suppress notices.

The audit regression uses the real auditor and a local advisory Git database.
It demonstrates that vulnerable lockfiles fail, clean lockfiles pass,
maintenance notices remain visible, and local ignore configuration is rejected.
