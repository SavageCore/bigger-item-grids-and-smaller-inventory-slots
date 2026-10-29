# AGENTS.md

## Versioning

- The mod version is `VERSION` in the `Makefile` and `version` in
  `mod.toml`, wherever each exists. Where both exist they carry the
  same version - keep them in sync on every bump.
- A version bump is its own commit: it touches only those version
  lines, nothing else, and its message is the version alone
  (`0.2.2`, never `Bump version...`). Never bump the version as part
  of a fix or feature change.
