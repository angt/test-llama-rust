#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use clap::Parser;
use std::ffi::CString;

// A simple llama.cpp version in rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Path to the model gguf file
    #[arg(short = 'm', long = "model")]
    model: String,

    // number of tokens to predict
    #[arg(short = 'n', default_value_t = 32)]
    n_predict: u32,

    // prompt to generate text from
    prompt: String,
}

fn print_tokens(vocab: *const llama_vocab, tokens: &[llama_token])
{
    let mut buf = [0u8; 128];

    for &token in tokens {
        let n = unsafe {
            llama_token_to_piece(vocab, token.into(), buf.as_mut_ptr(), buf.len() as _, 0, true)
        };
        if n < 0 {
            panic!("Failed to decode token")
        }
        print!("{}", String::from_utf8_lossy(&buf[0..n as usize]));
    }
}

fn main() {
    let args = Args::parse();

    println!("Argument -m: {}", args.model);
    println!("Prompt: {}", args.prompt);

    let model_path = CString::new(args.model).unwrap();
    let prompt = CString::new(args.prompt).unwrap();

    unsafe { // too big ? :P
        llama_backend_init();

        let model_params = llama_model_default_params();
        let model = llama_model_load_from_file(model_path.as_ptr(), model_params);

        if model.is_null() {
            panic!("Failed to load model");
        }
        let vocab = llama_model_get_vocab(model);

        if vocab.is_null() {
            panic!("Failed to get vocab");
        }
        let n_prompt = -llama_tokenize(vocab,
            prompt.as_ptr(), prompt.count_bytes() as _,
            0 as _, 0, true, true);

        if n_prompt <= 0 {
            panic!("Failed to tokenize");
        }
        let mut prompt_tokens = vec![0 as llama_token; n_prompt as _];

        let result = llama_tokenize(vocab,
            prompt.as_ptr(), prompt.count_bytes() as _,
            prompt_tokens.as_mut_ptr(),
            n_prompt, true, true);

        if result < 0 {
            panic!("Failed to tokenize");
        }
        println!("Nombre de tokens : {}", n_prompt);

        let mut ctx_params = llama_context_default_params();
        ctx_params.n_ctx = n_prompt as u32 + args.n_predict - 1;
        ctx_params.n_batch = n_prompt as u32;

        let ctx = llama_init_from_model(model, ctx_params);

        if ctx.is_null() {
            panic!("Failed to init llama");
        }
        let sparams = llama_sampler_chain_default_params();
        let smpl = llama_sampler_chain_init(sparams);

        if smpl.is_null() {
            panic!("Failed to init sampler");
        }
        let greedy_sampler = llama_sampler_init_greedy();

        if greedy_sampler.is_null() {
            panic!("Failed to init greedy sampler");
        }
        llama_sampler_chain_add(smpl, greedy_sampler);

        print_tokens(vocab, &prompt_tokens);

        let mut batch = llama_batch_get_one(prompt_tokens.as_ptr() as _, prompt_tokens.len() as _);

        let limit: llama_token = n_prompt as llama_token + args.n_predict as llama_token;
        let mut n_pos = 0 as llama_token;

        while batch.n_tokens + n_pos < limit {
            if llama_decode(ctx, batch) != 0 {
                panic!("Failed to decode");
            }
            n_pos += batch.n_tokens as i32;
            let mut next = llama_sampler_sample(smpl, ctx, -1);

            if llama_vocab_is_eog(vocab, next) {
                break;
            }
            print_tokens(vocab, &[next]);
            batch = llama_batch_get_one(&mut next, 1);
        }
        llama_sampler_free(smpl);
        llama_free(ctx);
        llama_model_free(model);
    }
}
