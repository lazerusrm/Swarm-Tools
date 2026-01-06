// Build script to download semantic embedding model and ONNX Runtime (Windows)
// This runs automatically during `cargo build` and `cargo install`

use std::path::PathBuf;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    let models_dir = out_path.join("models");
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir).unwrap();
    }

    // Download tokenizer (small file)
    let tokenizer_url =
        "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json";
    let tokenizer_path = models_dir.join("tokenizer.json");

    if !tokenizer_path.exists() {
        println!("Downloading tokenizer.json...");
        if let Err(e) = download_file(tokenizer_url, &tokenizer_path) {
            println!("Warning: Could not download tokenizer.json: {}", e);
        } else {
            println!("Downloaded tokenizer.json");
        }
    }

    // Download config.json (very small)
    let config_url =
        "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json";
    let config_path = models_dir.join("config.json");

    if !config_path.exists() {
        println!("Downloading config.json...");
        if let Err(e) = download_file(config_url, &config_path) {
            println!("Warning: Could not download config.json: {}", e);
        } else {
            println!("Downloaded config.json");
        }
    }

    // Download ONNX model (large - 86MB)
    // Skip in CI to avoid timeouts - models will be downloaded at runtime
    let onnx_url = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx";
    let onnx_path = models_dir.join("model.onnx");

    if !onnx_path.exists() {
        // Check if we're in CI environment
        if std::env::var("CI").is_ok() {
            println!("Skipping ONNX model download in CI (will download at runtime)");
        } else {
            println!("Downloading ONNX model (this may take a moment)...");
            if let Err(e) = download_file(onnx_url, &onnx_path) {
                println!("Warning: Could not download ONNX model: {}", e);
                println!("Semantic engine will use fallback embeddings");
            } else {
                println!("Downloaded ONNX model");
            }
        }
    }

    // On Windows, download ONNX Runtime DLL
    #[cfg(windows)]
    {
        let ort_dll_url = "https://cdn.pyke.io/onnxruntime-win-x64-1.20.0/onnxruntime.dll";
        let ort_dll_path = models_dir.join("onnxruntime.dll");

        if !ort_dll_path.exists() {
            if std::env::var("CI").is_ok() {
                println!("Skipping ONNX Runtime DLL download in CI");
            } else {
                println!("Downloading ONNX Runtime DLL for Windows...");
                if let Err(e) = download_file(ort_dll_url, &ort_dll_path) {
                    println!("Warning: Could not download ONNX Runtime DLL: {}", e);
                } else {
                    println!("Downloaded ONNX Runtime DLL");
                }
            }
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn download_file(url: &str, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.get(url).send()?;
    if !response.status().is_success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("HTTP error: {}", response.status()),
        )));
    }

    let mut file = std::fs::File::create(path)?;
    let content = response.bytes()?;
    std::io::Write::write_all(&mut file, &content)?;
    Ok(())
}
