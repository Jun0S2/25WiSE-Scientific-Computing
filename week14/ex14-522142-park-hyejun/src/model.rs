use burn::{
    module::Module,
    nn::{
        conv::{Conv2d, Conv2dConfig},                       // for convolutional layers
        pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig}, // for pooling layers
        Linear, LinearConfig, Relu,                         // for fully connected layers and activation
        PaddingConfig2d,                                    // for padding configuration
        Dropout, DropoutConfig,                             // for dropout
        BatchNorm, BatchNormConfig,                         // for batch normalization
    },
    tensor::backend::Backend,
    tensor::Tensor,
};

#[derive(Module, Debug)]
pub struct HexDigitModel<B: Backend> {
    conv1: Conv2d<B>, // [1,32]
    bn1: BatchNorm<B, 2>, // add batch norm layer
    conv2: Conv2d<B>, // [32,64]
    bn2: BatchNorm<B, 2>,
    conv3: Conv2d<B>,   // [64,128]
    bn3: BatchNorm<B, 2>,
    fc1: Linear<B>, // 128*7*7 = 6272 -> 512
    fc2: Linear<B>,
    // fc3: Linear<B>,
    dropout: Dropout,
    activation: Relu,
    pool: AdaptiveAvgPool2d,
}

impl<B: Backend> HexDigitModel<B> {
    pub fn new(device: &B::Device) -> Self {
        // 1. Convolution Layer (16 -> 32 -> 64 channels)
        let conv1 = Conv2dConfig::new([1, 16], [3, 3]).with_padding(PaddingConfig2d::Same).init(device);
        let bn1 = BatchNormConfig::new(16).init(device);

        let conv2 = Conv2dConfig::new([16, 32], [3, 3]).with_padding(PaddingConfig2d::Same).init(device);
        let bn2 = BatchNormConfig::new(32).init(device);

        let conv3 = Conv2dConfig::new([32, 64], [3, 3]).with_padding(PaddingConfig2d::Same).init(device);
        let bn3 = BatchNormConfig::new(64).init(device);

        // 2. Pooling data (final output size 7x7)
        let pool = AdaptiveAvgPool2dConfig::new([7, 7]).init();

        // 3. Classifier Layer (Fully Connected Layers)
        let fc1 = LinearConfig::new(3136, 512).init(device); // channel 64 * 7 * 7 = 3136
        let fc2 = LinearConfig::new(512, 16).init(device);

        let dropout = DropoutConfig::new(0.1).init(); // 0.1 dropout rate because model is small
        let activation = Relu::new();

        Self { conv1, bn1, conv2, bn2, conv3, bn3, fc1, fc2, dropout, activation, pool }
    }

    // Forward function - defines the data flow through the network
    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 2> {
        let mut x = self.activation.forward(self.bn1.forward(self.conv1.forward(x)));   // [batch, 16, 28, 28]
        x = self.activation.forward(self.bn2.forward(self.conv2.forward(x)));           // [batch, 32, 28, 28]
        x = self.activation.forward(self.bn3.forward(self.conv3.forward(x)));           // [batch, 64, 28, 28]

        x = self.pool.forward(x);                                       // [batch, 64, 7, 7]
        let [batch_size, channels, height, width] = x.dims();           // unpack dimensions
        let x = x.reshape([batch_size, channels * height * width]);     // flatten to [batch, 3136]

        let x = self.activation.forward(self.fc1.forward(x));           // [batch, 512]
        let x = self.dropout.forward(x);                                // apply dropout
        
        self.fc2.forward(x)                                             // [batch, 16]         

    }
}