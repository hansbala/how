use anyhow::Result;
use directories::ProjectDirs;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::num::NonZeroU32;

const MODEL_BYTES: &[u8] = include_bytes!("../model_resources/qwen2.5-coder-0.5b-instruct-q4_k_m.gguf");

unsafe extern "C" fn quiet_log_callback(
    _level: llama_cpp_sys_2::ggml_log_level,
    _text: *const std::os::raw::c_char,
    _user_data: *mut std::os::raw::c_void,
) {
    // Intentionally empty. This eats the logs.
}

fn get_cached_model_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "hansbala", "how")
        .ok_or_else(|| anyhow::anyhow!("Unable to determine cache directory"))?;
    
    let cache_dir = proj_dirs.cache_dir();
    fs::create_dir_all(cache_dir)?;
    
    let model_path = cache_dir.join("qwen2.5-coder-0.5b-instruct-q4_k_m.gguf");
    
    if !model_path.exists() {
        eprintln!("Extracting model to cache...");
        let mut file = fs::File::create(&model_path)?;
        file.write_all(MODEL_BYTES)?;
        eprintln!("Model cached successfully.");
    }
    
    Ok(model_path)
}

fn main() -> Result<()> {
    unsafe {
        llama_cpp_sys_2::llama_log_set(Some(quiet_log_callback), std::ptr::null_mut());
    }

    // Get arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: how <your request>");
        return Ok(());
    }
    let user_request = args[1..].join(" ");

    // Initialize Backend
    let backend = LlamaBackend::init()?;

    // Get cached model path (will extract if needed)
    let model_path = get_cached_model_path()?;

    let model_params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)?;

    // Create Context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(1024)); // Increased context slightly for the longer prompt

    let mut ctx = model.new_context(&backend, ctx_params)?;

    // Format Prompt (One-Shot Technique)
    // We give it ONE example (list files -> ls -la) so it mimics the "raw command" style.
    let prompt = format!(
        "<|im_start|>system\nYou are a command line tool. Output ONLY the raw shell command. Do not use markdown code blocks.<|im_end|>\n<|im_start|>user\nlist all files in detail\n<|im_end|>\n<|im_start|>assistant\nls -la\n<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
        user_request
    );

    // Tokenize
    let tokens_list = model.str_to_token(&prompt, AddBos::Always)?;

    if tokens_list.len() > 1024 {
         eprintln!("Prompt too long.");
         return Ok(());
    }

    let mut batch = LlamaBatch::new(1024, 1);
    let last_index = tokens_list.len() as i32 - 1;

    for (i, token) in tokens_list.iter().enumerate() {
        batch.add(*token, i as i32, &[0], i as i32 == last_index)?;
    }
    
    ctx.decode(&mut batch)?;

    // Generation Loop
    let mut n_cur = batch.n_tokens();
    let mut output_tokens = 0;
    let mut sampler = LlamaSampler::greedy();
    
    // Buffer for the full text
    let mut response_buffer = String::new();

    loop {
        let next_token = sampler.sample(&ctx, batch.n_tokens() - 1);

        if next_token == model.token_eos() || output_tokens > 200 {
            break;
        }
        
        let output_text = model.token_to_str(next_token, Special::Tokenize)?;
        
        // Append to buffer instead of printing immediately
        response_buffer.push_str(&output_text);

        batch.clear();
        batch.add(next_token, n_cur, &[0], true)?;
        ctx.decode(&mut batch)?;

        n_cur += 1;
        output_tokens += 1;
    }

    // Cleanup Logic
    // 1. Remove "```bash", "```sh", "```" from the start
    // 2. Remove "```" from the end
    // 3. Trim whitespace
    let cleaned_response = response_buffer
        .trim()
        .trim_start_matches("```bash")
        .trim_start_matches("```sh")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    println!("{}", cleaned_response);

    Ok(())
}

