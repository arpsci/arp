# ARP Engineering Guide

This book explains the application as it exists in the current repository.

ARP is a local-first Rust desktop app built on `eframe`/`egui`. It starts a Tokio runtime, optionally starts a small embedded Rocket server, renders a graph-based agent workspace, and runs multi-agent conversations through Ollama. Runs are persisted under `runs/`, timing metrics are written to JSONL, and Python virtual environments are managed under `runtimes/python/`.

This guide covers:

- process startup and the egui shell,
- the conversation runner and sidecar execution model,
- run manifests, append-only event ledgers, and metrics,
- managed Python runtimes and traced task execution,
- vault gating, outbound HTTP policy, and embedded web endpoints,
- environment variables and operational defaults.

The page set intentionally matches the book structure already in the repository. Use the chapters in order for onboarding, or jump straight to Configuration when you need an exact runtime knob.
