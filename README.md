# arp 

Agents Research Platform for HCI and Cognitive Sciences.


### Overview

- Multi-agent conversations
- Prompt design and injection
- Reproducibility of inference
- Local and field-first architecture



### Building

```sh
# Development: Builds to 'target/debug/'
cargo run

# Distribution: Builds to 'target/release/'
cargo build --release
```

### Current Architecture

Agents are rows: **Manager**, **Worker**, **Evaluator**, **Researcher**, plus **Topic** presets. Rows wire to each other via dropdowns (e.g. worker→manager/topic). **Start** saves a manifest and runs Ollama loops: workers with a topic are **paired in id order** (two workers ⇒ dialogue, one ⇒ solo loop). Evaluators/researchers are **sidecars** on each turn when active. **Stop** ends all loops.

### Communication

JSON `POST` to `CONVERSATION_HTTP_ENDPOINT` (default `http://localhost:3000/`). Conversation events include `sender_id`, `receiver_id`, `topic`, `message`, …; evaluator/researcher events use `evaluator_name` / sentiment (researcher uses `sentiment` like `references:<topic>`). RFC3339 UTC timestamps. Runs may include `experiment_id`, `run_id`, `manifest_version`.

### Reproducible Runs

`Start` writes `runs/<experiment_id>/<run_id>/manifest.json` (`manifest_version = "2.0.0"`): runtime settings plus a **flat agent snapshot** (each node `config` holds its links—no separate edge list). Settings: export manifest, load manifest + run (read-only), bundle zip.


### Dependencies

- rust-adk
- eframe
- egui-phosphor
- egui-snarl = { path = "crates/egui-snarl" }

