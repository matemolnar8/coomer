Answer questions about this review concern.

## Steps

When no question follows this paste, explain this review concern.

1. Resolve the pinned SHAs.
   Run `git rev-parse --verify 2c167c094b21500cfcbf4733064d83f6a69dc4f7` and `git rev-parse --verify d0eefb26770e07ed783d616269cc994820c9238f` in this repository.
   Done when both objects exist.

2. Load the hunks.
   A hunk ref is a pointer into the live git diff at the pinned SHAs.
   For each hunk ref, run `git diff --find-renames 2c167c094b21500cfcbf4733064d83f6a69dc4f7 d0eefb26770e07ed783d616269cc994820c9238f -- <path>` and keep the hunk whose header matches the @@ range.
   Done when every hunk ref has a matching live hunk.

3. Answer from live git.
   Read those hunks. Use the why and the what as interpretation. Live git wins when they disagree.
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

Review concern 03 of 05: Install comprehende (`comprehende`)

Part: Agent setup

The why:

`AGENTS.md` sends PR reviews through `comprehende`, so the repo installs it [(session)](source:s1).

The what:

`npx skills add matemolnar8/comprehende` wrote the vendored skill and its `skills-lock.json` entry, and `.claude/skills` symlinks to `../.agents/skills` so Claude Code finds it. You can skip the vendored skill text: it is an exact copy of the published skill.

Depends on:
- 02 Remove mattpocock skills (`remove-mattpocock`)

Hunk refs for this concern:
- .agents/skills/comprehende/SKILL.md
- .agents/skills/comprehende/references/example.md
- .agents/skills/comprehende/references/review.schema.json
- .claude/skills
- skills-lock.json