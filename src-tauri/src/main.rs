#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ncm;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
struct FileInfo {
    name: String,
    size: u64,
}

#[derive(Serialize, Deserialize)]
struct ConvertResult {
    format: String,
    output_path: String,
}

#[derive(Serialize, Deserialize)]
struct AudioConvertResult {
    output_path: String,
    progress: u32,
}

#[tauri::command]
fn get_file_info(path: String) -> Result<FileInfo, String> {
    let path = PathBuf::from(path);
    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(FileInfo {
        name,
        size: metadata.len(),
    })
}

#[tauri::command]
fn scan_ncm_files(path: String) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map(|ext| ext == "ncm").unwrap_or(false) {
            files.push(path.to_string_lossy().to_string());
        }
    }
    Ok(files)
}

#[tauri::command]
fn scan_audio_files(path: String, extensions: Vec<String>) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if extensions.contains(&ext_str) {
                files.push(path.to_string_lossy().to_string());
            }
        }
    }
    Ok(files)
}

#[tauri::command]
fn convert_ncm(input_path: String, output_dir: Option<String>) -> Result<ConvertResult, String> {
    let input = PathBuf::from(&input_path);
    
    let output_directory = match output_dir {
        Some(dir) => PathBuf::from(dir),
        None => input.parent().unwrap_or(&PathBuf::from(".")).to_path_buf(),
    };
    
    let crypt = ncm::NeteaseCrypt::new(&input_path)?;
    let (format, output_path) = crypt.dump(&output_directory)?;
    
    Ok(ConvertResult {
        format,
        output_path: output_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn convert_audio(
    app: tauri::AppHandle,
    input_path: String, 
    output_format: String, 
    output_dir: Option<String>
) -> Result<AudioConvertResult, String> {
    let input = PathBuf::from(&input_path);
    
    let output_directory = match output_dir {
        Some(dir) => PathBuf::from(dir),
        None => input.parent().unwrap_or(&PathBuf::from(".")).to_path_buf(),
    };
    
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    
    let output_path = output_directory.join(format!("{}.{}", stem, output_format));
    
    let shell = app.shell();
    let sidecar = shell.sidecar("ffmpeg").map_err(|e| e.to_string())?;
    
    let output = sidecar
        .args([
            "-y",
            "-i", &input_path,
            "-vn",
            "-acodec", get_codec(&output_format),
            "-q:a", "2",
            &output_path.to_string_lossy()
        ])
        .output()
        .await
        .map_err(|e| format!("FFmpeg 执行失败: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg 转换失败: {}", stderr));
    }
    
    Ok(AudioConvertResult {
        output_path: output_path.to_string_lossy().to_string(),
        progress: 100,
    })
}

#[tauri::command]
async fn check_ffmpeg(app: tauri::AppHandle) -> Result<String, String> {
    let shell = app.shell();
    match shell.sidecar("ffmpeg") {
        Ok(_) => Ok("FFmpeg 已内嵌".to_string()),
        Err(_) => Err("未找到 FFmpeg".to_string())
    }
}

fn get_codec(format: &str) -> &str {
    match format {
        "mp3" => "libmp3lame",
        "wav" => "pcm_s16le",
        "flac" => "flac",
        "aac" => "aac",
        "ogg" => "libvorbis",
        "wma" => "wmav2",
        "m4a" => "aac",
        "opus" => "libopus",
        "aiff" => "pcm_s16be",
        "ape" => "ape",
        _ => "copy",
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_file_info,
            scan_ncm_files,
            scan_audio_files,
            convert_ncm,
            convert_audio,
            check_ffmpeg
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
