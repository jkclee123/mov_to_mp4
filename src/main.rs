use std::fs;
use std::path::Path;
use std::process::Command;
use colored::*;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use thiserror::Error;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::io::{self, Write};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("FFmpeg error: {0}")]
    FFmpegError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Path error: {0}")]
    PathError(String),
}

fn main() -> Result<(), AppError> {
    print!("Do you want to delete MOV files after conversion? (yes/no): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let delete_after = input.trim().to_lowercase() == "yes" || input.trim().to_lowercase() == "y";
    
    let ffmpeg_path = get_ffmpeg_path()?;
    let mov_filenames = get_all_mov()?;
    let total = mov_filenames.len();
    println!("Found {} MOV files to process", total);
    
    let mut success = 0;
    let mut failed = 0;
    
    let pb = ProgressBar::new(total as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}\n\
                  Current: {msg}")
        .unwrap()
        .progress_chars("█▓▒░"));
    
    for mov_filename in mov_filenames {
        let display_name = Path::new(&mov_filename)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(&mov_filename);
        
        pb.set_message(format!("Converting: {}", display_name));
        
        // Create a flag to control the progress update thread
        let should_continue = Arc::new(AtomicBool::new(true));
        let should_continue_clone = Arc::clone(&should_continue);
        
        // Start a thread to update the progress bar
        let pb_clone = pb.clone();
        let handle = thread::spawn(move || {
            while should_continue_clone.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
                pb_clone.tick();
            }
        });

        match convert_mov_to_mp4(&mov_filename, &ffmpeg_path) {
            Ok(_) => {
                if delete_after {
                    let _ = remove_mov(&mov_filename);
                }
                success += 1;
                pb.println(format!("{} Successfully converted: {}", "✓".green(), display_name));
            }
            Err(e) => {
                failed += 1;
                pb.println(format!("{} Failed to convert {}: {}", "✗".red(), display_name, e));
            }
        }
        
        // Signal the thread to stop and wait for it
        should_continue.store(false, Ordering::Relaxed);
        handle.join().unwrap();
        pb.inc(1);
    }
    
    pb.finish_with_message("Conversion complete");
    
    println!("\nSummary:");
    println!("{} Total files processed", total);
    println!("{} Successfully converted", success);
    println!("{} Failed conversions", failed);
    
    Ok(())
}

fn get_all_mov() -> Result<Vec<String>, AppError> {
    let mov_dir = Path::new("mov");
    let mut mov_filenames = Vec::new();

    for entry in fs::read_dir(mov_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }

        if let Some(extension) = path.extension() {
            if extension.eq_ignore_ascii_case("mov") {
                if let Some(name_str) = path.file_name().and_then(|f| f.to_str()) {
                    let mov_file = mov_dir.join(name_str);
                    mov_filenames.push(mov_file.to_str()
                        .ok_or_else(|| AppError::PathError("Invalid path".to_string()))?
                        .to_string());
                }
            }
        }
    }

    Ok(mov_filenames)
}

/// Converts a MOV file to MP4 format using FFmpeg
/// 
/// This function handles the conversion process with optimized settings for 
/// fast conversion while maintaining good quality. It uses hardware acceleration
/// when available on the platform.
fn convert_mov_to_mp4(mov_filename: &str, ffmpeg_path: &str) -> Result<(), AppError> {    
    // Create mp4 directory if it doesn't exist
    let mp4_dir = Path::new("mp4");
    if !mp4_dir.exists() {
        fs::create_dir(mp4_dir)?;
    }

    // Prepare output file path with same name but mp4 extension
    let file_name = Path::new(mov_filename)
        .file_name()
        .ok_or_else(|| AppError::PathError("Invalid filename".to_string()))?;
    let mp4_file = mp4_dir.join(file_name).with_extension("mp4");
    
    // ==========================================
    // FFmpeg conversion argument configuration
    // ==========================================
    
    // 1. Configure hardware acceleration based on OS
    let hw_accel_args: Vec<&str> = if cfg!(target_os = "macos") {
        vec!["-hwaccel", "videotoolbox"]
    } else if cfg!(target_os = "windows") {
        vec!["-hwaccel", "dxva2"]
    } else if cfg!(target_os = "linux") {
        vec!["-hwaccel", "vaapi", "-vaapi_device", "/dev/dri/renderD128"]
    } else {
        vec![]
    };

    // 2. Initialize the arguments list
    let mut args = Vec::new();
    
    // 3. Add hardware acceleration (if available for platform)
    args.extend_from_slice(&hw_accel_args);
    
    // 4. Specify input file
    args.extend_from_slice(&["-i", mov_filename]);
    
    // 5. Configure video codec settings
    //    - libx264: High quality H.264 encoder
    //    - faster preset: Good balance between speed and quality
    //    - CRF 23: Default quality setting (lower = better quality)
    args.extend_from_slice(&[
        "-c:v", "libx264", 
        "-preset", "faster",
        "-crf", "23",
    ]);
    
    // 6. Configure audio codec settings
    //    - AAC: Industry standard audio codec
    //    - 128k bitrate: Good quality for most audio sources
    args.extend_from_slice(&[
        "-c:a", "aac", 
        "-b:a", "128k",
    ]);
    
    // 7. Optimize for performance with multithreading
    //    - 0 threads means auto-detect available CPU cores
    args.extend_from_slice(&["-threads", "0"]);
    
    // 8. Specify output file
    args.push(mp4_file.to_str()
        .ok_or_else(|| AppError::PathError("Invalid MP4 path".to_string()))?);
    
    // ==========================================
    // Execute FFmpeg conversion command
    // ==========================================
    
    let output = Command::new(ffmpeg_path)
        .args(&args)
        .output()?;
    
    // Check if conversion was successful
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::FFmpegError(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

fn get_ffmpeg_path() -> Result<String, AppError> {
    if let Ok(path) = which::which("ffmpeg") {
        return Ok(path.to_str()
            .ok_or_else(|| AppError::PathError("Invalid FFmpeg path".to_string()))?
            .to_string());
    }

    let ffmpeg_path = if cfg!(target_os = "windows") {
        PathBuf::from("bin/ffmpeg/ffmpeg.exe")
    } else {
        PathBuf::from("bin/ffmpeg/ffmpeg")
    };

    if !ffmpeg_path.exists() {
        return Err(AppError::FFmpegError(format!(
            "FFmpeg binary not found. Please ensure it's either:\n\
            1. Installed and available in your system PATH, or\n\
            2. Located at {:?}",
            ffmpeg_path
        )));
    }

    ffmpeg_path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::PathError("Failed to convert FFmpeg path to string".to_string()))
}

fn remove_mov(mov_filename: &str) -> Result<(), AppError> {
    fs::remove_file(mov_filename)?;
    Ok(())
}