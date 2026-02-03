mod splitter;
mod model;
mod train;
mod infer;

use std::path::Path;
use regex::Regex;
use burn::module::Module; // load_record
use burn::record::Recorder; // recorder.load

// Constants
const BASE_URL: &str = "https://www.zib.de/userpage/koch/scscans/";
const TEST_FOLDER: &str = "test/";
// const PAGE_FOLDER: &str = "page/";  // cannot access
// const DATA_FOLDER: &str = "data/";  // cannot access

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("PROGRAM STARTS");

    // Step 1: Dynamically fetch image list from the server
    println!("Step 1: Fetching image list from server...");
    let image_names = get_image_list().await?;

    if image_names.is_empty() {
        println!("⚠️  No images found on the server. Using fallback images.");

        // Fallback hardcoded list
        let fallback_images = vec![
            "hd1.png".to_string(), "hd2.png".to_string(), "hd3.png".to_string(),
            "hd4.png".to_string(), "hd5.png".to_string(), "hd6.png".to_string(),
            "hd7.png".to_string(), "hd8.png".to_string()
        ];
        download_images(&fallback_images).await?;
    } else {
        println!("Found {} images on the server", image_names.len());
        download_images(&image_names).await?;
    }
    println!("======================================");

    // Step 2: Split images
    println!("Step 2: Splitting images...");
    splitter::split_all_images()?;
    println!("======================================");

    // Step 3: Train the model
    println!("Step 3: Training model...");
    train::train_model()?; // trains and saves the model to a file
    println!("======================================");

    // Step 4: Test inference
    println!("Step 4: Running multiple inference tests...");
    
    // 1. Set up device for inference
    #[cfg(feature = "wgpu")]
    let device = burn::backend::wgpu::WgpuDevice::BestAvailable;
    #[cfg(not(feature = "wgpu"))]
    let device = burn::backend::ndarray::NdArrayDevice::Cpu;

    // 2. Call the saved best_model file from train step
    // Define backend type conditionally
    #[cfg(feature = "wgpu")]
    type MyBackend = burn::backend::Wgpu;
    #[cfg(not(feature = "wgpu"))]
    type MyBackend = burn::backend::NdArray<f32>;

    let recorder = burn::record::CompactRecorder::new(); // recorder for Model loading
    let model: model::HexDigitModel<MyBackend> = model::HexDigitModel::new(&device)
        .load_record(recorder.load("data/model/best_model".into(), &device)?);

    // 3. Get test files from split folder
    let entries: Vec<_> = std::fs::read_dir("data/split")?
        .filter_map(|e| e.ok())
        .collect();

    let test_files = entries.iter().take(10); // select first 10 for simplicity
    
    let mut correct_count = 0;
    for entry in test_files {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        let expected = filename.chars().next().unwrap(); 
        
        // 4. call run inference to test out the model
        let predicted = infer::run_inference(&model, &path, &device)?;
        let is_correct = expected == predicted;
        if is_correct { correct_count += 1; }
        
        println!("File: {:<25} | Expected: {} | Predicted: {} | {}", 
                filename, expected, predicted, if is_correct { "✓" } else { "✗" });
    }
    
    println!("Test Result: {}/10", correct_count);
    println!("Program completed successfully!");
    Ok(())
}

// Dynamically fetch image list from server
async fn get_image_list() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = format!("{}{}", BASE_URL, TEST_FOLDER);

    println!("Connecting to server: {}", url);

    // Fetch HTML page
    let html = match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                response.text().await?
            } else {
                println!("⚠️  Server response error: {}", response.status());
                return Ok(Vec::new());
            }
        }
        Err(e) => {
            println!("⚠️  Failed to connect to server: {}", e);
            return Ok(Vec::new());
        }
    };

    // Extract PNG links using regex
    let re = Regex::new(r#"href="([^">]+\.png)"#)?;
    let mut images: Vec<String> = re.captures_iter(&html)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter(|name| {
            // Filter out parent directories or special files
            !name.starts_with("../") &&
            !name.starts_with("?") &&
            name.ends_with(".png")
        })
        .collect();

    // Remove duplicates and sort
    images.sort();
    images.dedup();

    println!("Discovered images:");
    for (i, img) in images.iter().enumerate() {
        println!("  {}. {}", i + 1, img);
    }

    Ok(images)
}

// Image download function
async fn download_images(image_names: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::time::Duration;

    // Create download folder
    fs::create_dir_all("downloads")?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let total = image_names.len();

    for (i, image_name) in image_names.iter().enumerate() {
        let url = format!("{}{}{}", BASE_URL, TEST_FOLDER, image_name);
        let file_path = format!("downloads/{}", image_name);

        // Check if already exists
        if Path::new(&file_path).exists() {
            println!("[{}/{}] Already exists: {}", i + 1, total, image_name);
            continue;
        }

        println!("[{}/{}] Downloading: {}", i + 1, total, image_name);

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let bytes = response.bytes().await?;

                    // Save file
                    fs::write(&file_path, bytes)?;
                    println!(
                        "Saved: {} ({} bytes)",
                        image_name,
                        std::fs::metadata(&file_path)?.len()
                    );
                } else {
                    println!("  ⚠️  Download failed: HTTP {}", response.status());
                }
            }
            Err(e) => {
                println!("  ⚠️  Download error: {}", e);
            }
        }

        // Wait to avoid overloading the server (0.5s)
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Print downloaded files
    println!("Download complete. Contents of downloads/ folder:");

    if let Ok(entries) = fs::read_dir("downloads") {
        let mut count = 0;
        for entry in entries.filter_map(Result::ok) {
            if let Ok(metadata) = entry.metadata() {
                println!(
                    "  - {} ({} bytes)",
                    entry.file_name().to_string_lossy(),
                    metadata.len()
                );
                count += 1;
            }
        }
        println!("Total {} files", count);
    }

    Ok(())
}
