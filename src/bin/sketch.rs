// use tch;

fn main() {
    let device = tch::Device::cuda_if_available();
    println!("hello sketch on device: {device:?}")
}
