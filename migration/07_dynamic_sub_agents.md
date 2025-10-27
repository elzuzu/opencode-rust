# Ticket 7: Implement Dynamic Sub-Agent Orchestration

**Task:** Bring the Rust port (`opencode-rust/`) to feature parity with the TypeScript implementation's dynamic, atomic sub-agent flow.

---

## 1. Business Goals & User Impact
- **Goal:** Allow the Rust binary to delegate focussed tasks to short-lived sub-agents whose prompts, models, and tools are determined at call time.
- **Why:** In the shipped TypeScript app (`packages/opencode`), the "task" tool (see `packages/opencode/src/tool/task.ts`) spins up child sessions for @mentions or plan/build delegations. The Rust port must support the same UX so downstream clients (CLI, TUI, ACP, Desktop) can continue launching focussed sub-sessions without losing parity.

---

## 2. Behaviour in the Existing (TypeScript) Codebase
Use these modules as the source of truth:
- `packages/opencode/src/session/prompt.ts`
  - Computes the **effective model** via `resolveModel`, defaulting to the parent's selection when `agent.model` is absent (`inherit`).
  - Builds the **system prompt** by combining repo context (CLAUDE.md, agents config), session-specific reminders, and agent-provided prompt sections. Everything is emitted as tagged sections (`<ROLE>`, `<OBJECTIVE>`, etc.).
  - Resolves **tools** dynamically: when an agent omits `tools`, the runtime inherits everything registered with `ToolRegistry`, including MCP bridges. Explicit tool lists turn into allow-lists.
  - Launches **subtasks** when a command is marked `subtask` or when a subagent triggers `TaskTool`, using `SessionPrompt.prompt` recursively to create child sessions.
- `packages/opencode/src/tool/task.ts`
  - Parses runtime JSON definitions, narrows to subagents (`mode !== "primary"`), and launches a new session with its own message stream, while streaming summary metadata back to the parent tool call.
  - Applies per-agent model overrides and merges tool allow/deny flags with parent defaults.
- `packages/opencode/src/agent/agent.ts`
  - Persists agent metadata (`mode`, `model`, `tools`, prompt snippets) and merges project, user, and session-level definitions (including the CLI `--agents` JSON overrides).

---

## 3. Required Rust Architecture
1. **Dynamic Agent Specs**
   - Introduce `AgentSpec` + `Budgets` structs (or equivalent) under `opencode-rust/src/agent/`. These must be constructed at runtime from:
     - Project configuration (port of `agent.ts` merge logic).
     - User overrides.
     - CLI-provided JSON definitions (see §4).
   - Default semantics: `model: inherit` (no override) and `tools: None` (inherit the parent tool registry).

2. **Prompt Composer**
   - Add a `PromptBuilder` service (e.g., `opencode-rust/src/session/prompt_builder.rs`) that mirrors the TypeScript sections:
     - `<ROLE>`: concatenated `prompt_sections` from the spec.
     - `<OBJECTIVE>`: active task / command string.
     - `<PROJECT_RULES>`: compiled from `CLAUDE.md`, repo `AGENTS.md`, migration docs, etc.
     - `<CONSTRAINTS>` + `<REPORT_FORMAT>`: default guards/budgets exactly like the TS prompt builder.
   - Must consume existing Rust helpers for repo discovery (`watcher`, `util`) or add them as part of the ticket.

3. **Model & Tool Resolution**
   - Extend the orchestrator (likely `opencode-rust/src/session` or equivalent) with:
     - `resolve_model(spec: &AgentSpec, parent: &ModelHandle) -> ModelHandle` implementing the `inherit` semantics.
     - `resolve_tools(spec: &AgentSpec, parent_tools: &[Arc<dyn Tool>]) -> Vec<Arc<dyn Tool>>` that either clones the parent's list or filters by allow-list.
   - Ensure MCP servers that are already registered in Rust remain available unless explicitly removed.

4. **Runtime JSON Agents**
   - Update the CLI command that initiates runs (`opencode-rust/src/cli/cmd/run.rs` & parsing glue) to accept `--agents-json <path-or-inline>`, parse it with `serde_json`, and merge the resulting map into the in-memory agent registry before execution.
   - Precedence rules must match TypeScript: CLI definitions > project `.opencode/agents` > user config.

5. **Atomic Sub-agent Execution**
   - Implement `spawn_subagent(...)` in a new module (e.g., `opencode-rust/src/session/subagent.rs`). Responsibilities:
     - Build the prompt via `PromptBuilder`.
     - Resolve model/tools with the helpers above.
     - Emit lifecycle events over the existing event bus (`flume`/`tokio::sync::mpsc` channel used by the Rust runtime) mirroring `AgentEvent::Started/Completed` semantics.
     - Invoke the LLM client once per invocation, never reusing conversational history; only return the summary/artefact payload to the parent.
     - Enforce token/tool timeouts defined in `Budgets`.
   - Integrate with the orchestrator so that plan/build flows (and future `task` tool invocations) can run several subagents concurrently via `tokio::task::JoinSet`.

6. **Session & Tool Plumbing**
   - Create Rust equivalents of:
     - Creating a child session when the Task tool is executed.
     - Streaming summary metadata back to the parent while the child runs.
   - Ensure that parent sessions do not receive child transcripts, only the summarised metadata, consistent with the TS implementation (`TaskTool.execute`).

7. **Testing Hooks**
   - Provide unit/integration tests under `opencode-rust/tests/` covering:
     - JSON agent parsing precedence.
     - Prompt assembly with repository context fixtures.
     - Model/tool inheritance and allow-list behaviour.
     - Subagent task spawning and event emission (mock the LLM client).

---

## 4. Acceptance Criteria (Rust Specific)
- [ ] `AgentSpec` supports `inherit` defaults and merges all configuration layers.
- [ ] `PromptBuilder` emits deterministic tagged sections matching the TypeScript prompt layout for equivalent inputs.
- [ ] CLI exposes `--agents-json` (or similar) with precedence identical to the JS implementation.
- [ ] Subagent runs use isolated LLM calls, honour `Budgets`, and surface only summaries back to the parent session/event stream.
- [ ] Parallel execution is supported by spawning multiple subagents with `JoinSet` (or equivalent) while sharing the parent tool registry safely.
- [ ] Comprehensive tests document the expected flows and guard against regressions.

---

## 5. Reference Material
- TS orchestration: `packages/opencode/src/session/prompt.ts`
- Task tool delegation: `packages/opencode/src/tool/task.ts`
- Agent registry & merge logic: `packages/opencode/src/agent/agent.ts`
- Existing Rust scaffolding: `opencode-rust/src/agent`, `opencode-rust/src/tool`, `opencode-rust/src/session`

Capture key behaviours during implementation so the Rust port preserves the runtime flexibility relied upon by the production OpenCode clients.
