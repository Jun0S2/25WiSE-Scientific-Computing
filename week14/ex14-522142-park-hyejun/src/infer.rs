use burn::{
    prelude::*,
    tensor::Tensor,
};

use crate::model::HexDigitModel;
use crate::Path;

const CHAR_LABELS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F'
];

/// main.rs will call this method to test out the model
pub fn run_inference<B: Backend>(
    model: &HexDigitModel<B>,
    path: &Path,
    device: &B::Device,
) -> Result<char, Box<dyn std::error::Error>> {
    // 1. Read Image
    let pixels = crate::train::read_pbm_file(path)?;

    // 2. convert to tensor [Batch: 1, Channel: 1, H: 28, W: 28]
    let tensor = Tensor::<B, 2>::from_data(
        burn::tensor::TensorData::new(pixels, [1, 784]),
        device,
    )
    .reshape([1, 1, 28, 28]);

    // 3. call model forward
    let output = model.forward(tensor);

    // 4. get predicted index
    // get the first value from the data iterator because argmax returns a tensor
    let predicted_idx = output.argmax(1).into_data().iter::<i32>().next().unwrap() as usize;
    // let predicted_idx = output.argmax(1).into_scalar() as i32 as usize;
    
    if predicted_idx < CHAR_LABELS.len() {
        Ok(CHAR_LABELS[predicted_idx])  // return corresponding char label
    } else {
        Ok('?')
    }
}
