use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

mod ollama;

/// Sentinel error message when the user stops inference or starts a new run.
pub const OLLAMA_STOPPED_MSG: &str = "ollama inference stopped";

/// When live epoch differs from `captured_epoch`, streaming stops cooperatively (user Stop).
pub type OllamaStopEpoch = (Arc<AtomicU64>, u64);

pub async fn fetch_ollama_models() -> Result<Vec<String>> {
    ollama::fetch_models().await
}

pub async fn send_to_ollama(
    instruction: &str,
    input: &str,
    limit_token: bool,
    num_predict: &str,
    model_override: Option<&str>,
    stop_epoch: Option<OllamaStopEpoch>,
) -> Result<String> {
    send_to_ollama_with_context(
        instruction,
        input,
        limit_token,
        num_predict,
        model_override,
        stop_epoch,
    )
    .await
}

pub async fn send_to_ollama_with_context(
    instruction: &str,
    input: &str,
    limit_token: bool,
    num_predict: &str,
    model_override: Option<&str>,
    stop_epoch: Option<OllamaStopEpoch>,
) -> Result<String> {
    let runner_ctx =
        ollama::build_runner_context(instruction, limit_token, num_predict, model_override).await?;
    ollama::print_context_preview(input);
    if let Some((epoch, caught)) = &stop_epoch {
        if epoch.load(Ordering::SeqCst) != *caught {
            return Err(anyhow::anyhow!(OLLAMA_STOPPED_MSG));
        }
    }
    ollama::run_prompt_streaming(runner_ctx, input, false, stop_epoch).await
}

pub async fn test_ollama(model_override: Option<&str>) -> Result<String> {
    let runner_ctx = ollama::build_runner_context(
        "You are a helpful assistant running locally via Ollama.",
        false,
        "",
        model_override,
    )
    .await?;
    let input = "Hello, how are you?";
    println!("Input: {}", input);
    ollama::run_prompt_streaming(runner_ctx, input, true, None).await
}
