# Architecture

ARP is organized around a thin desktop shell and a stateful orchestration core.

## High-level modules

- `src/main.rs`: creates the Tokio runtime, optionally launches Rocket, and boots the egui app.
- `src/ui/mod.rs`: owns the top-level `AMSAgentsApp`, vault gate, theme/font setup, and per-frame routing into the workspace UI.
- `src/agents/`: owns the graph state, run orchestration, conversation loops, prompt assembly, and evaluator/researcher sidecars.
- `src/ollama/`: fetches model tags and streams chat inference while recording timing metrics.
- `src/run/`: writes `manifest.json`, `events.jsonl`, and `summary.json` for each run.
- `src/python/`: creates and manages portable venvs, plus traced Python task execution helpers.
- `src/vault.rs`: verifies the master password and provides an encrypted in-memory blob container.
- `src/web/mod.rs`: enforces outbound HTTP guardrails, posts optional webhooks, and serves the embedded API when enabled.
- `src/metrics/`: writes inference and turn timing events to JSONL through a replaceable sink.

## Data roots

- `runs/`: one directory per run bundle, including manifests, ledgers, summaries, and Python task sidecars.
- `runtimes/python/`: the runtime registry plus one directory per managed venv.
- `metrics/timings.jsonl`: default sink for inference and turn timing events.
- `runs/.master_hash`: optional default location for the vault Argon2id PHC hash.

## Core object wiring

The main runtime object is `AMSAgents`, constructed by `AMSAgentsApp`. It keeps together the state that has to span UI frames and async run execution:

- the Tokio `Handle`,
- `AppState`, which currently owns the live metrics sink,
- Ollama host/model settings,
- outbound HTTP policy flags,
- active run context and manifest,
- the append-only event ledger for the current run,
- the node-graph workspace state,
- chat bridge channels used to forward agent turns into the Overview chat UI.

That split is deliberate: `AMSAgentsApp` is responsible for frame lifecycle and vault gating, while `AMSAgents` owns the application behavior that must survive across frames and background tasks.
