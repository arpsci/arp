# Runs and Audit Trail

ARP stores each execution under `runs/<experiment_id>/<run_id>/`.

## Run manifest

`manifest.json` is the primary description of a run. It includes:

- manifest and app version metadata,
- generated `experiment_id` and `run_id`,
- a graph signature derived from canonical runtime and graph data,
- runtime settings such as Ollama host/model, HTTP endpoint, history size, replay mode, HTTP policy flags, and metrics config,
- a graph snapshot containing one record per node row, including serialized node config.

The current manifest version is `2.0.0`.

## Event ledger

Every run can also have an append-only `events.jsonl` ledger. The ledger is opened when a run starts and writes monotonically increasing `event_id` values. Each envelope stores:

- run identity,
- event type and timestamp,
- optional node-global-id and model,
- SHA-256 hashes of the logical input and output,
- a JSON payload.

Common event families in the current code include:

- `system.run_started`
- `system.run_stopped`
- `dialogue.start`
- `dialogue.turn`
- `transport.http`
- `transport.http_blocked`
- `python_task.started`
- `python_task.finished`

When a run is finalized, `summary.json` is written next to the ledger. It records event counts, total events, transport success/failure counters, first and last timestamps, and SHA-256 digests for `manifest.json` and `events.jsonl` when those files exist.

## Metrics alongside audit data

Timing metrics are not stored in the run bundle by default. They are written through the metrics sink to `metrics/timings.jsonl` unless the user changes that path in Settings.

That JSONL contains two event families:

- `inference_timing` for Ollama requests, including TTFT and token counts when available,
- `turn_timing` for dialogue gaps and speaker/receiver sequencing.

## Overview chat persistence

The Overview chat remains a separate persistence path from the run ledger.

- the chat UI stores room/message history through the Overview chat store,
- the audit module keeps append-only chat audit output for that subsystem,
- active conversation runs can mirror agent messages into the currently active room over the in-process channel bridge.

In short: the run ledger is the reproducibility trail for orchestration, while Overview chat persistence is the UI-facing conversational record.
