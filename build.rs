// Build script to download semantic embedding model and ONNX Runtime (Windows)
// This runs automatically during `cargo build` and `cargo install`

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::PathBuf::from(&out_dir);

    // Create models directory in OUT_DIR for build
    let models_dir = out_path.join("models");
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir).unwrap();
    }

    // Download tokenizer if not present
    let tokenizer_url =
        "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json";
    let tokenizer_path = models_dir.join("tokenizer.json");

    if !tokenizer_path.exists() {
        println!("Downloading tokenizer.json...");
        download_file(tokenizer_url, &tokenizer_path);
        println!("Downloaded tokenizer.json");
    }

    // Download config.json
    let config_url =
        "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json";
    let config_path = models_dir.join("config.json");

    if !config_path.exists() {
        println!("Downloading config.json...");
        download_file(config_url, &config_path);
        println!("Downloaded config.json");
    }

    // Download ONNX model
    let onnx_url = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx";
    let onnx_path = models_dir.join("model.onnx");

    if !onnx_path.exists() {
        println!("Downloading ONNX model (this may take a moment)...");
        download_file(onnx_url, &onnx_path);
        println!("Downloaded ONNX model");
    }

    // On Windows, download ONNX Runtime DLL
    #[cfg(windows)]
    {
        let ort_dll_url = "https://cdn.pyke.io/onnxruntime-win-x64-1.20.0/onnxruntime.dll";
        let ort_dll_path = models_dir.join("onnxruntime.dll");

        if !ort_dll_path.exists() {
            println!("Downloading ONNX Runtime DLL for Windows...");
            download_file(ort_dll_url, &ort_dll_path);
            println!("Downloaded ONNX Runtime DLL");
        }
    }

    // Print location for debugging
    println!("Model files will be copied to: {:?}", models_dir);

    // Tell Cargo to rerun this script if these files change
    println!("cargo:rerun-if-changed=build.rs");
}

fn download_file(url: &str, path: &std::path::PathBuf) {
    let response = reqwest::blocking::get(url).expect(&format!("Failed to download {}", url));
    let mut file =
        std::fs::File::create(path).expect(&format!("Failed to create {}", path.display()));
    let mut content = response.bytes().expect("Failed to read response");
    std::io::Write::write_all(&mut file, &mut content).expect("Failed to write file");
}
