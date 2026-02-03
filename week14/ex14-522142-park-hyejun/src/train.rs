use burn::{
    data::{
        dataloader::{DataLoaderBuilder, batcher::Batcher},
        dataset::Dataset,
    },
    module::Module,
    optim::{AdamConfig, Optimizer},
    record::{CompactRecorder, Recorder},
    tensor::{backend::Backend, Int, Tensor, TensorData},
};
use crate::model::HexDigitModel;
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle}; // for progress bar

// ---  Dataset  ---
pub struct HexDataset {
    images: Vec<Vec<f32>>,  // pixel data for each image
    labels: Vec<usize>,     // corresponding labels (1-F)
}

impl HexDataset {
    pub fn new() -> Self {
        let mut images = Vec::new();
        let mut labels = Vec::new();
        let split_dir = "data/split";
        
        // Create dataset from images in data/split
        if let Ok(entries) = std::fs::read_dir(split_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                // filter only .pbm files
                if path.extension().and_then(|s| s.to_str()) == Some("pbm") {
                    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) { 
                        // Use the first character of the filename as the label (0-F)
                        if let Some(char_label) = filename.chars().next() {
                            let label = match char_label {
                                '0'..='9' => char_label as usize - '0' as usize,
                                'A'..='F' => char_label as usize - 'A' as usize + 10,
                                _ => continue,
                            };
                            // Read PBM file and store pixel data and label
                            if let Ok(img_data) = read_pbm_file(&path) {
                                images.push(img_data);
                                labels.push(label);
                            }
                        }
                    }
                }
            }
        }
        println!("Dataset loaded: {} samples", images.len());
        Self { images, labels }
    }
}

// Burn 0.14 Dataset Trait Implementation
impl Dataset<(Vec<f32>, usize)> for HexDataset {
    fn get(&self, index: usize) -> Option<(Vec<f32>, usize)> { // return image data and label
        let img = self.images.get(index)?.clone(); // clone to return owned data
        let label = *self.labels.get(index)?; // get label
        Some((img, label))          
    }
    fn len(&self) -> usize { self.images.len() } // total number of samples
}

// Reads a PBM file and returns pixel data as Vec<f32>
pub fn read_pbm_file(path: &Path) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // 1. Load PBM image as grayscale (processed by spliiter to be 28x28)
    let img = image::open(path)?.to_luma8();
    
    // 2. Convert pixels to f32 and normalize to [-1.0, 1.0] 
    let pixels: Vec<f32> = img.pixels()
        .map(|p| {
            if p[0] > 0 { 
                1.0  // 1 : writing part
            } else { 
                -1.0 // 0 : paper part
            }
        })
        .collect();

    Ok(pixels)
}

// --- 2. Define Batcher  ---
#[derive(Clone)]
pub struct HexBatcher<B: Backend> {
    device: B::Device,
}

// Data Structure for a single training step (Batch)
#[derive(Clone, Debug)]
pub struct HexBatch<B: Backend> {
    pub images: Tensor<B, 4>,   
    pub targets: Tensor<B, 1, Int>,
}

impl<B: Backend> Batcher<(Vec<f32>, usize), HexBatch<B>> for HexBatcher<B> {
    fn batch(&self, items: Vec<(Vec<f32>, usize)>) -> HexBatch<B> { // items: Vec of (image data, label)
        // Convert each image to a 3D tensor [1, 28, 28]
        let images = items.iter()
            .map(|(img, _)| {
                Tensor::<B, 1>::from_data(TensorData::new(img.clone(), [784]), &self.device) // flatten to [784]
                .reshape([1, 28, 28])   // reshape to [1, 28, 28]
            })
            .collect::<Vec<_>>();
        
        let targets = items.iter() // extract labels
            .map(|(_, label)| *label as i32)
            .collect::<Vec<_>>();

        // Create 4D batch tensor [Batch, 1, 28, 28] by stacking 3D tensors
        let images = Tensor::<B, 3>::stack(images, 0); 
        let targets = Tensor::<B, 1, Int>::from_data(
            TensorData::from(targets.as_slice()), 
            &self.device
        );

        HexBatch { images, targets }
    }
}

// --- 3. Train Model ---
pub fn train_model() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Backend Type Aliasing (Compile-time definition)
    // This defines the specific implementation of the Backend trait to be used.
    // It determines how tensors are represented in memory and which math libraries are linked.
    #[cfg(feature = "wgpu")]
    type MyBackend = burn::backend::Autodiff<burn::backend::Wgpu>;
    #[cfg(not(feature = "wgpu"))]
    type MyBackend = burn::backend::Autodiff<burn::backend::NdArray<f32>>;

    // 2. Device Instantiation (Runtime allocation)
    // This creates an instance of the specific hardware resource (CPU or GPU).
    // It tells the Backend exactly where the actual computation should be executed.
    #[cfg(feature = "wgpu")]
    let device = burn::backend::wgpu::WgpuDevice::BestAvailable;
    #[cfg(not(feature = "wgpu"))]
    let device = burn::backend::ndarray::NdArrayDevice::Cpu;

    // Usage Example:
    // We pass the 'Type' to the struct definition: HexDigitModel<MyBackend>
    // We pass the 'Instance' to the initializer: .init(&device)
    println!("Using device: {:?}", device);

    let dataset = HexDataset::new();
    let num_samples = dataset.len();
    let batch_size = 64;

    let batcher = HexBatcher::<MyBackend> { device: device.clone() };

    // Dataloader : creates batches from the dataset using the batcher
    // shuffle -> added to improved my model performance( randomizes the order of samples each epoch )
    let dataloader = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .shuffle(42)
        .build(dataset);

    let mut model = HexDigitModel::<MyBackend>::new(&device); // Model Initialization
    // Optimizer Initialization (Adam with Weight Decay) 0> with weight decay is used to prevent overfitting
    let mut optim = AdamConfig::new() 
        .with_weight_decay(Some(burn::optim::decay::WeightDecayConfig::new(1e-4))) 
        .init();

    let num_epochs = 100; // maximum epochs -> will be stopped early if no improvement
    let lr = 1e-3; // tried 5e-4, and 2e-3 but 1e-3 worked best in my tests

    // Early Stopping Variables
    let mut best_accuracy = 0.0;
    let mut no_improvement = 0;
    let patience = 10;  // max epochs without improvement (10 -> 5)
    let recorder = CompactRecorder::new();

    println!("Step 3: Training model...");

    for epoch in 1..=num_epochs {
        let mut iter = dataloader.iter();
        let mut epoch_loss = 0.0;
        let mut count = 0;
        let mut correct = 0;

        let pb = ProgressBar::new((num_samples / batch_size) as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [Epoch {pos}/{len}] [{wide_bar:.cyan/blue}] {msg} ({eta})")?
            .progress_chars("#>-"));
        
        while let Some(batch) = iter.next() {
            let output = model.forward(batch.images);
            
            // calculate number of correct predictions
            let predicted = output.clone().argmax(1).reshape([batch.targets.dims()[0]]);
            let n_correct = predicted.equal(batch.targets.clone())
                .int().sum().into_scalar() as i32;
            correct += n_correct;

            // calculate loss
            // CrossEntropyLoss for multi-class classification
            let loss = burn::nn::loss::CrossEntropyLossConfig::new()
                .init(&device).forward(output, batch.targets);
            
            epoch_loss += loss.clone().into_scalar();
            
            // Backpropagation and optimization step
            let grads = loss.backward();
            let grads = burn::optim::GradientsParams::from_grads(grads, &model);
            model = optim.step(lr, model, grads);
            
            count += batch_size;
            pb.inc(1); // increment progress bar
            pb.set_message(format!("Loss: {:.4}", loss.into_scalar())); // update message
        }
        
        pb.finish_and_clear();
        
        // Calculate average loss and accuracy for the epoch after all batches
        let avg_loss = epoch_loss / (num_samples as f32 / batch_size as f32);
        let accuracy = (correct as f32 / count as f32) * 100.0;
        
        println!("Epoch {}/{} | Loss: {:.4} | Acc: {:.2}%", epoch, num_epochs, avg_loss, accuracy);

        // Early Stopping Check
        if accuracy > best_accuracy {
            best_accuracy = accuracy;
            no_improvement = 0;
            // Save best model (simple_model.mpk)
            recorder.record(model.clone().into_record(), "data/model/best_model".into())?;
            println!("Updated best accuracy. Saved Model");
        } else {
            no_improvement += 1;
        }

        if no_improvement >= patience {
            println!("[✓] Stop epochs - no improvements observed : Total Run {} epochs (highest: {:.2}%)", patience, best_accuracy);
            break;
        }
    }

    Ok(())
}