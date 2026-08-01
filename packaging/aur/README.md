# AUR packaging (`fat`)

Published: https://aur.archlinux.org/packages/fat

## Install

```bash
yay -S fat
# or
paru -S fat
```

## Automatic publish

Every **GitHub Release** (and manual **Actions → Publish release**) runs [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml):

1. Publishes crates.io (`CARGO_REGISTRY_TOKEN`)
2. Bumps `packaging/aur/PKGBUILD` + `.SRCINFO`
3. Pushes the package to the AUR (`AUR_SSH_PRIVATE_KEY`)

### One-time setup

```bash
gh secret set AUR_SSH_PRIVATE_KEY < ~/.ssh/aur_synara
gh secret set CARGO_REGISTRY_TOKEN <<<"$TOKEN"
```

The AUR public key must already be on your AUR account (same key as optionMusic / opsh).

### Day-to-day

```bash
git tag -a v0.1.2 -m "fat 0.1.2"
git push origin v0.1.2
gh release create v0.1.2 --title "fat 0.1.2" --generate-notes
# → Actions publishes crates.io + AUR
```

Manual re-run: **Actions → Publish release → Run workflow**.

## Local publish (fallback)

```bash
./packaging/aur/publish.sh           # push current packaging/
./packaging/aur/publish.sh 0.1.2     # bump + push
```

Uses `~/aur/fat` and `~/.ssh/aur_synara` (override with `AUR_SSH_KEY=` / `AUR_DIR=`).
