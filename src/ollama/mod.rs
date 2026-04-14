use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

mod client;

/// Sentinel error message when the user stops inference or starts a new run.
pub const OLLAMA_STOPPED_MSG: &str = "ollama inference stopped";

/// When live epoch differs from `captured_epoch`, streaming stops cooperatively (user Stop).
pub type OllamaStopEpoch = (Arc<AtomicU64>, u64);

pub async fn fetch_ollama_models(ollama_host: &str) -> Result<Vec<String>> {
    client::fetch_models(ollama_host).await
}

pub async fn send_to_ollama(
    ollama_host: &str,
    instruction: &str,
    input: &str,
    limit_token: bool,
    num_predict: &str,
    model_override: Option<&str>,
    stop_epoch: Option<OllamaStopEpoch>,
) -> Result<String> {
    let runner_ctx = client::build_runner_context(
        ollama_host,
        instruction,
        limit_token,
        num_predict,
        model_override,
    )
    .await?;
    client::print_context_preview(input);
    if let Some((epoch, caught)) = &stop_epoch {
        if epoch.load(Ordering::SeqCst) != *caught {
            return Err(anyhow::anyhow!(OLLAMA_STOPPED_MSG));
        }
    }
    client::run_prompt_streaming(runner_ctx, input, false, stop_epoch).await
}

pub async fn test_ollama(ollama_host: &str, model_override: Option<&str>) -> Result<String> {
    let runner_ctx = client::build_runner_context(
        ollama_host,
        "You are a helpful assistant running locally via Ollama.",
        false,
        "",
        model_override,
    )
    .await?;
    let input = "Hello, how are you?";
    println!("Input: {}", input);
    client::run_prompt_streaming(runner_ctx, input, true, None).await
}
