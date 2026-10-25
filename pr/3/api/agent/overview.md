Answer questions about this git change.

## Steps

When no question follows this paste, explain this change.

1. Resolve the pinned SHAs.
   Run `git rev-parse --verify 2c167c094b21500cfcbf4733064d83f6a69dc4f7` and `git rev-parse --verify d0eefb26770e07ed783d616269cc994820c9238f` in this repository.
   Done when both objects exist.

2. Choose the relevant review concerns.
   Read Review concerns. Fetch a concern file only when that concern is relevant to the question.
   Done when every concern the question touches has its markdown loaded.

3. Answer from live git.
   Follow those files. Use the why and the what as interpretation. Live git wins when they disagree.
   When you show code, quote the live git lines.
   Done when the answer quotes the live code.

## Pin

Repository: coomer
Origin: git@github.com:matemolnar8/coomer.git

base (merge-base)  2c167c094b21500cfcbf4733064d83f6a69dc4f7

head               d0eefb26770e07ed783d616269cc994820c9238f

Named refs at pin: main ... chore/agents-md-rewrite

Read the diff:

git diff --find-renames 2c167c094b21500cfcbf4733064d83f6a69dc4f7 d0eefb26770e07ed783d616269cc994820c9238f

Commits:
- d0eefb2 Port pages skill for hosted PR reviews
- 1e54420 Install comprehende skill
- 94f85cc Fix clippy warnings
- 6605d4f Remove mattpocock skills and rewrite AGENTS.md

Sources:
- transcript Claude Code session · Oct 25 Remove the mattpocock skills and their artifacts, write a concise AGENTS.md based on cigster, comprehende, and vitadeck, set up cargo and keep everything valid, install comprehende since AGENTS.md uses it, port the pages setup, open a PR.

The title:

Replace mattpocock skills with comprehende and pages, rewrite AGENTS.md

The why:

Máté no longer uses the mattpocock skills and wants a concise `AGENTS.md` modeled on cigster, comprehende, and vitadeck [(session)](source:s1).

The what (small):

The branch deletes the 21 mattpocock skills and their `docs/agents/` output, rewrites `AGENTS.md`, installs `comprehende`, and ports vitadeck's `pages` skill for hosted PR reviews. A separate part fixes four clippy warnings in `src/`.

## Review concerns

### 01 AGENTS.md rewrite (`agents-md`)

`AGENTS.md` follows vitadeck's sections (Glossary, How it works, Footguns, Hit every surface, Verifying, Pull requests) and keeps the four existing coomer conventions under Taste.

[groups/agents-md.md](groups/agents-md.md)

### 02 Remove mattpocock skills (`remove-mattpocock`)

Every deleted file under `.agents/skills/` and `docs/agents/` came from `mattpocock/skills` or its setup skill, and `skills-lock.json` drops their 21 entries.

[groups/remove-mattpocock.md](groups/remove-mattpocock.md)

### 03 Install comprehende (`comprehende`)

`npx skills add matemolnar8/comprehende` wrote the vendored skill and its `skills-lock.json` entry, and `.claude/skills` symlinks to `../.agents/skills` so Claude Code finds it. You can skip the vendored skill text: it is an exact copy of the published skill.

Depends on:
- 02 Remove mattpocock skills (`remove-mattpocock`)

[groups/comprehende.md](groups/comprehende.md)

### 04 Port pages skill (`pages`)

`pages-review.ts` publishes a comprehende export to `pr/<n>/` on `gh-pages`, and `pages-reviews.yml` runs its `prune` when a PR closes and daily for sites older than 30 days. The script matches vitadeck except for the temp dir prefix and fallback git identity.

Depends on:
- 03 Install comprehende (`comprehende`)

[groups/pages.md](groups/pages.md)

### 05 Clippy fixes (`clippy`)

`draw_session` takes the existing `RenderSnapshot` instead of nine arguments, which removes the identity `NSPoint` to `CGPoint` conversions in `overlay.rs`; three nested `if let`s become let chains.

[groups/clippy.md](groups/clippy.md)