# Python Runtime Management

Managed Python environments live under `runtimes/python/`.

## Registry and runtime model

The runtime subsystem keeps a persistent registry at `runtimes/python/python_runtimes.json`. Each entry is a `PythonRuntime` with:

- a generated id such as `pyrt_ab12cd34`,
- a human label,
- the resolved Python version from the created venv,
- the runtime root path,
- provenance fields such as `created_at` and `created_by`,
- the declarative `PythonRuntimeSpec`,
- a lifecycle state.

Lifecycle states are:

- `Active`
- `Deprecated`
- `Deleted`

Deleting a runtime removes the directory and preserves only the registry record for traceability.

## Creation flow

`create_runtime()` creates a fresh venv and records the full transcript in `create.log`. The current flow is:

1. create the runtime directory,
2. run `<base_interpreter> -m venv`,
3. resolve the created venv's Python version,
4. upgrade `pip`,
5. install any requested packages,
6. run any post-install commands,
7. write `requirements.lock` from `pip freeze`.

The default on-disk layout is:

- `runtimes/python/python_runtimes.json`
- `runtimes/python/<runtime_id>/create.log`
- `runtimes/python/<runtime_id>/requirements.lock`
- the full venv contents under the same directory.

## UI surface today

The current Python panel is focused on environment management rather than task authoring. From the UI you can:

- create a new environment,
- view the active runtime details,
- install additional packages into that runtime,
- open an external terminal running the venv Python REPL,
- destroy the environment.

The panel stores one active runtime in UI state at a time.

## Task execution flow

The runtime module also exposes `launch_task()` for traced script execution inside a managed runtime. Given a run directory and active ledger, it will:

1. create `runs/<experiment>/<run>/python_tasks/<task_id>/`,
2. capture `stdout.log` and `stderr.log`,
3. write `meta.json` with command, env overrides, exit code, and timestamps,
4. emit `python_task.started` and `python_task.finished` ledger events.

Tasks are launched with ARP-specific environment variables injected:

- `ARP_EXPERIMENT_ID`
- `ARP_RUN_ID`
- `ARP_EVENTS_PATH`

That gives downstream Python code a direct path back to the enclosing run bundle.
