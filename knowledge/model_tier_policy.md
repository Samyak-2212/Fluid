<!-- version: 2 -->
# Model Tier Policy

Different agents run on different models. Quality varies significantly.
This policy governs which model class does which work.

## Tier A (Claude Sonnet and GPT-5.4)

Reserved for:

- Architecture decisions
- Interface design between components
- Numerical method selection and validation
- Code review of Tier B output flagged as `[NEEDS_REVIEW]`
- Bug fixes marked `severity: critical` or `severity: arch-break` in bug_pool
- Writing coordinator prompts

Approved Tier A models for this repository:

- Claude Sonnet
- GPT-5.4

## Tier B (Gemini or similar)

Used for:

- Boilerplate implementation (getters, serialization, config parsing)
- Test writing from specifications
- Documentation generation
- Non-critical bug fixes (`severity: low`, `severity: medium`)
- File scaffolding

## Review Gate

Any Tier B output touching the following must be tagged `[NEEDS_REVIEW: claude]` in a comment
block at the top of the file, and added to `bug_pool/BUG_POOL.md` under section
`## Pending Claude Review` with severity `review`:

- Physics integrators
- Memory safety (unsafe blocks)
- wgpu pipeline setup
- ECS core
- Numerical solvers
- CUDA / ROCm FFI bridges

## Prompt and Knowledge File Protection

`ROOT_COORDINATOR.md`, all `coordinators/*/PROMPT.md` files, and all files in `knowledge/`
are Tier A only. Tier B models may read them but must never modify them.

If a Tier B model identifies a necessary change, it must:
- File it in `bug_pool/BUG_POOL.md` under `## Prompt/Knowledge Changes`
- Mark severity `review`
- Leave the original file untouched

Any commit touching these files must include `[TIER_A_REVIEW]` in the commit message.
C7 audits these commits.

## knowledge_b/ Protocol

Tier B models write observations to `knowledge_b/`. Rules:

- One file per observation, named `<agent_id>_<timestamp>_<topic>.md`
- Write facts only: "function X is in file Y", "crate Z at version N is used"
- Never write conclusions, architectural recommendations, or interface designs
- Tag every entry with agent_id and timestamp
- Tier A reads `knowledge_b/` skeptically — treat as raw field notes, not ground truth
- Tier A must independently verify any entry before acting on it, or mark it `[UNVERIFIED]`
- Once Tier A verifies and promotes an entry to `knowledge/`, delete the `knowledge_b/` entry
  and commit the deletion

## Claude Usage Budget

Claude Sonnet has a weekly limit in the IDE. Coordinators must batch `[NEEDS_REVIEW]` items
and submit them together, not one-by-one.

- C7 owns the review queue and batching
- Batching cadence: C7 submits review batches at most once per day, or when the queue exceeds
  10 items — whichever comes first
- If the weekly budget is exhausted, all `[NEEDS_REVIEW]` items are queued and held until the
  following week
- C7 must log budget exhaustion in `bug_pool/BUG_POOL.md` under `## Process Violations`
- Never use Claude for boilerplate, test scaffolding, or config files
