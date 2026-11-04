# Optional Sem context as a local knowledge source

Litho already accepts Markdown/JSON through `knowledge.local_docs`. You can use
[Sem](https://github.com/Ataraxy-Labs/sem) to prepare a small, explicit source
snapshot for a module you want the documentation agents to investigate. This is
an offline input recipe, not a live MCP integration or a replacement for Litho's
source-file tools.

Install Sem separately. From the Git checkout being documented, run the following
Bash example, replacing `parse_config` with a symbol in that checkout:

```bash
set -eu
export SEM_LOCAL=1 SEM_NO_NETWORK=1 SEM_NO_TELEMETRY=1
mkdir -p docs
snapshot=$(mktemp)
trap 'rm -f "$snapshot"' EXIT
{
  printf '# Source context snapshot\n\nGit HEAD: '
  git rev-parse HEAD
  printf '\nWorking-tree status at capture:\n\n```text\n'
  git status --short
  printf '```\n\nSem version: '
  sem --version
  printf '\nRequested entity context:\n\n```json\n'
  sem find parse_config --context --budget 2000 --json
  printf '\n```\n'
} > "$snapshot"
mv "$snapshot" docs/sem-context.md
```

The temporary file avoids replacing a prior snapshot if lookup fails. Inspect
the output before using it: a successful command can return no matches, and
ambiguous names may need `--in src/module/`. The budget may omit related entities;
the snapshot is evidence to investigate, not a complete or compiler-verified
call graph. Sem may write local indexes/cache data. Its local controls are not
an OS network sandbox.

In the existing `litho.toml`, enable local docs if needed and add a category (do
not duplicate an existing `[knowledge.local_docs]` table):

```toml
[knowledge.local_docs]
enabled = true
watch_for_changes = true

[[knowledge.local_docs.categories]]
name = "general"
description = "Selected source context; verify against current source files"
paths = ["docs/sem-context.md"]
```

Run `deepwiki-rs sync-knowledge --force`, then your usual documentation-generation
command with that same configuration and project. An empty `target_agents` list
(the default) makes this available to all agents; configure specific agents if
only a subset needs the context.

Regenerate and sync after code changes. The recorded HEAD alone cannot identify
dirty source contents, and `watch_for_changes` watches the snapshot file rather
than rerunning Sem. Treat source text as data, not instructions. The snapshot may
contain proprietary code and enter model prompts and cached knowledge, so use
only providers approved for the checkout. Keep generated snapshots out of version
control unless you intentionally want to publish them.

To opt out, remove the category from your configuration and use your normal
knowledge-cache lifecycle. This recipe does not change presets, enable source
editing, or send code to Parallel's web-search endpoint.
