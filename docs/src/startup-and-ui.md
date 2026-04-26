# Startup and UI Flow

## Startup path

The binary startup is still compact, but there are a few important details worth keeping in mind:

1. Create a multi-threaded Tokio runtime.
2. Keep that runtime alive on a background thread.
3. Optionally launch the embedded Rocket server when `AMS_WEB_ENABLED` is enabled.
4. Start the egui native app with a `900x840` initial viewport and the window title `arp-cogsci`.

The UI shell receives only the Tokio handle; everything else is built inside `AMSAgentsApp::new`.

## Frame loop responsibilities

`AMSAgentsApp` owns three kinds of work on each frame:

1. Apply shell presentation once.
   Today that means the Catppuccin Latte theme plus the Phosphor icon font.
2. Enforce the vault gate.
   If the app is locked, the frame renders only the unlock screen and returns early.
3. Render the live workspace.
   Once unlocked, the app shows a top lock bar, refreshes Ollama models on first use, and delegates the main body to the graph workspace renderer.

The app-level UI state is also kept here. It includes:

- Ollama model list and test status,
- the manifest export path field,
- the Python panel form state and background-operation results.

## UI ownership model

The shell is intentionally thin.

- `AMSAgentsApp` handles frame lifecycle, unlock/lock behavior, and one-time shell setup.
- `AMSAgents` renders the actual workspace and owns behavior such as run control, model settings, metrics settings, HTTP policy toggles, Python runtime actions, and Overview chat forwarding.

That boundary matters when changing the UI: if a feature needs to survive across async tasks or active runs, it usually belongs in `AMSAgents`, not in the ephemeral frame-local widget code.
