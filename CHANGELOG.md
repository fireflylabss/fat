# Changelog

We follow [Semantic Versioning](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/). fat is a single CLI surface.

<details>
<summary>To see more about versioning, expand this.</summary>

Every version string starts with `v` (required), e.g. `v0.1.2`.

Here the installable surface is **CLI** (`fat`). Other Option apps swap in their own names the same way — e.g. **GNOME**, **Desktop**, **Web** — whatever you actually ship.

| Part | What you install | Example |
| --- | --- | --- |
| **CLI** | `fat` in the terminal | `v0.1.2` |

With one surface there is no `m` in the tag and no per-surface sections — just the version notes.

Each release heading is the version and date (`## v0.1.2 · 12/08/2026`); under it, a short summary ends with a plain sentence like: “This version was made for CLI on 12/08/2026 (v0.1.2).”

</details>

## v0.1.2 · 12/08/2026

Adopt optionSDK for color handling. This version was made for CLI on 12/08/2026 (v0.1.2).

- Added `optionSDK` as the first shared-family dependency.
- Use `option_sdk::color_enabled()` for `NO_COLOR` detection in auto color mode.
- Bumped crate version from `0.1.1` to `0.1.2`.
