#!/usr/bin/env bash
set -Eeuo pipefail

PROJECT_DIR="/home/gabriel/AEFireflyLabs/fat"
PROFILE_DIR="/home/gabriel/horizzon3507"

cd "$PROJECT_DIR"

# Remove apenas o .git vazio e protegido criado pelo ambiente.
# Se ele tiver arquivos, rmdir falha sem apagar nenhum dado.
if [[ -d .git ]] && ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  rmdir .git
fi

cargo fmt --check
cargo test

git init -b main
git add --all
git commit -m "Initial public release"

gh auth login -h github.com
gh repo create fireflylabss/fat \
  --public \
  --source=. \
  --remote=origin \
  --push \
  --description "A fast, syntax-aware cat alternative written in Rust"

cd "$PROFILE_DIR"
if ! rg -q 'github.com/fireflylabss/fat' README.md; then
  sed -i '/^### Utilities$/a\\
\
- 🧭 **[fat](https://github.com/fireflylabss/fat)** — Fast, syntax-aware cat alternative written in Rust' README.md
  git add README.md
  git commit -m "Add fat to profile README"
  git push
fi
