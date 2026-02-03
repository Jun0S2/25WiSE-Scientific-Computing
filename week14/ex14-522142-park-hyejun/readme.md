Architecture plan

```
root/
├── Cargo.toml # dependencies
├── src/
│ ├── main.rs
│ ├── splitter.rs # image splitter module
│ ├── model.rs # define neural nw
│ ├── train.rs # Train Module
│ └── infer.rs # Inference Module
├── data/
│ ├── raw/ # Original scanned images(hd1.png etc)
│ ├── split/ # Splitted text image
│ └── model/ # Saved Model file
└── downloads/ # Downloaded Images
```
