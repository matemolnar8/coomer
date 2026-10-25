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

Review concern 02 of 05: Remove mattpocock skills (`remove-mattpocock`)

Part: Agent setup

The why:

Máté no longer uses these skills [(session)](source:s1).

The what:

Every deleted file under `.agents/skills/` and `docs/agents/` came from `mattpocock/skills` or its setup skill, and `skills-lock.json` drops their 21 entries.

Hunk refs for this concern:
- .agents/skills/ask-matt/SKILL.md
- .agents/skills/code-review/SKILL.md
- .agents/skills/codebase-design/DEEPENING.md
- .agents/skills/codebase-design/DESIGN-IT-TWICE.md
- .agents/skills/codebase-design/SKILL.md
- .agents/skills/diagnosing-bugs/SKILL.md
- .agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh
- .agents/skills/domain-modeling/ADR-FORMAT.md
- .agents/skills/domain-modeling/CONTEXT-FORMAT.md
- .agents/skills/domain-modeling/SKILL.md
- .agents/skills/grill-me/SKILL.md
- .agents/skills/grill-with-docs/SKILL.md
- .agents/skills/grilling/SKILL.md
- .agents/skills/handoff/SKILL.md
- .agents/skills/implement/SKILL.md
- .agents/skills/improve-codebase-architecture/HTML-REPORT.md
- .agents/skills/improve-codebase-architecture/SKILL.md
- .agents/skills/prototype/LOGIC.md
- .agents/skills/prototype/SKILL.md
- .agents/skills/prototype/UI.md
- .agents/skills/research/SKILL.md
- .agents/skills/setup-matt-pocock-skills/SKILL.md
- .agents/skills/setup-matt-pocock-skills/domain.md
- .agents/skills/setup-matt-pocock-skills/issue-tracker-github.md
- .agents/skills/setup-matt-pocock-skills/issue-tracker-gitlab.md
- .agents/skills/setup-matt-pocock-skills/issue-tracker-local.md
- .agents/skills/setup-matt-pocock-skills/triage-labels.md
- .agents/skills/tdd/SKILL.md
- .agents/skills/tdd/mocking.md
- .agents/skills/tdd/tests.md
- .agents/skills/teach/GLOSSARY-FORMAT.md
- .agents/skills/teach/LEARNING-RECORD-FORMAT.md
- .agents/skills/teach/MISSION-FORMAT.md
- .agents/skills/teach/RESOURCES-FORMAT.md
- .agents/skills/teach/SKILL.md
- .agents/skills/to-spec/SKILL.md
- .agents/skills/to-tickets/SKILL.md
- .agents/skills/triage/AGENT-BRIEF.md
- .agents/skills/triage/OUT-OF-SCOPE.md
- .agents/skills/triage/SKILL.md
- .agents/skills/wayfinder/SKILL.md
- .agents/skills/writing-great-skills/GLOSSARY.md
- .agents/skills/writing-great-skills/SKILL.md
- docs/agents/domain.md
- docs/agents/issue-tracker.md
- docs/agents/triage-labels.md
- skills-lock.json