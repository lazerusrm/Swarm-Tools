#!/usr/bin/env pwsh
# Download semantic embedding model files for swarm-tools (Windows PowerShell)
# This script downloads the tokenizer needed for semantic embeddings

$MODEL_DIR = "models"
$TOKENIZER_URL = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json"

Write-Host "Downloading semantic embedding model files..." -ForegroundColor Green
Write-Host "Model directory: $MODEL_DIR" -ForegroundColor Cyan

New-Item -ItemType Directory -Force -Path $MODEL_DIR | Out-Null

Write-Host "Downloading tokenizer.json..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $TOKENIZER_URL -OutFile "$MODEL_DIR/tokenizer.json" -MaximumRetryCount 3 -RetryIntervalSec 2

Write-Host "" -ForegroundColor Green
Write-Host "Download complete!" -ForegroundColor Green
Write-Host "" -ForegroundColor Cyan
Write-Host "To use semantic embeddings, run:" -ForegroundColor Cyan
Write-Host "  cargo run --features semantic -- <your-command>" -ForegroundColor White
Write-Host "" -ForegroundColor Cyan
Write-Host "Or build a release binary:" -ForegroundColor Cyan
Write-Host "  cargo build --release --features semantic" -ForegroundColor White
