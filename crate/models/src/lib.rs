use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tch::IndexOp;

use crate::diffusion::{
    pipe::{Pipe, PipeConfig},
    text_transformer::Clip,
    tokenizer::Tokenizer,
    utils::{linspace, slerp, Bar},
};

pub mod diffusion;
pub mod llama;
pub mod sentiment;
pub mod summarize;
pub mod translation;

macro_rules! device {
    ($gpu: expr) => {
        tch::Device::Cuda($gpu)
    };
}

macro_rules! gpus {
    () => {
        0..(tch::Cuda::device_count() as usize)
    };
}

#[derive(Debug, Clone)]
pub struct Oneshot {
    pub prompt: String,
    pub height: i64,
    pub width: i64,
    pub seed: i64,
}

impl From<Oneshot> for PipeConfig {
    fn from(value: Oneshot) -> Self {
        let Oneshot {
            prompt,
            height,
            width,
            seed,
        } = value;

        PipeConfig {
            prompt: Some(prompt),
            height,
            width,
            seed,
            ..Default::default()
        }
    }
}

impl Oneshot {
    pub fn run(args: Oneshot) -> Result<(), &'static str> {
        println!("{args:#?}\n");

        tch::manual_seed(args.seed);
        let no_grad_guard = tch::no_grad_guard();
        let device = tch::Device::cuda_if_available();
        println!("{:?}", device);

        println!("- Build the Tokenizer\n");
        let tokenizer = Tokenizer::create(Default::default()).expect("cannot create tokenizer");
        let text = Clip::new(Some(device))
            .and_then(|clip| clip.run(&args.prompt, tokenizer))
            .expect("cannot encode prompt");

        println!("- Build pipe\n");
        let init = tch::Tensor::randn(
            [1, 4, args.height / 8, args.width / 8],
            (tch::Kind::Float, device),
        );

        let config = PipeConfig::from(args);
        let config = config.with_device(device);
        let mut pipe = Pipe::new(config).expect("cannot create pipe");

        println!("- Run pipe\n");
        pipe.diffuse(&init, &text, true, None)
            .expect("diffusion failed");

        drop(no_grad_guard);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    pub prompt: String,
    pub height: i64,
    pub width: i64,
    pub seed: i64,
    pub inference: i64,
}

impl From<Sequence> for PipeConfig {
    fn from(value: Sequence) -> Self {
        let Sequence {
            prompt,
            height,
            width,
            seed,
            inference,
        } = value;

        PipeConfig {
            prompt: Some(prompt),
            height,
            width,
            inference: Some(inference),
            seed,
            ..Default::default()
        }
    }
}

impl Sequence {
    pub fn run(args: Sequence) -> Result<(), &'static str> {
        assert!(args.height % 64 == 0);
        assert!(args.width % 64 == 0);
        tch::manual_seed(args.seed);
        let no_grad_guard = tch::no_grad_guard();
        let device = tch::Device::cuda_if_available();
        println!("{device:?}");

        let progress_bar = Bar::new(args.inference);
        progress_bar.print("build the pipe!".to_string());

        let tokenizer = Tokenizer::create(Default::default()).expect("cannot create tokenizer");
        let text = Clip::new(Some(device))
            .and_then(|clip| clip.run(&args.prompt, tokenizer))
            .expect("cannot encode prompt");

        let opts = (tch::Kind::Float, device);

        let config = PipeConfig::from(args.clone());
        let config = config.with_device(device);
        let mut pipe = Pipe::new(config.clone()).expect("new pipe failed");
        let init1 = tch::Tensor::randn([1, 4, config.height / 8, config.width / 8], opts);
        let init2 = tch::Tensor::randn([1, 4, config.height / 8, config.width / 8], opts);

        progress_bar.print("start the interpolation".to_string());
        for (k, x) in linspace(0, 1, args.inference) {
            progress_bar.set_position(k);
            let latents = slerp(x, &init1, &init2).expect("slerp failed");
            pipe.diffuse(&latents, &text, true, None)
                .expect("diffusion failed");
        }
        progress_bar.finish_with_message("generation");

        drop(no_grad_guard);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Parallel {
    pub prompt: String,
    pub height: i64,
    pub width: i64,
    pub seed: i64,
    pub inference: i64,
}

impl From<Parallel> for PipeConfig {
    fn from(value: Parallel) -> Self {
        let Parallel {
            prompt,
            height,
            width,
            seed,
            inference,
        } = value;

        PipeConfig {
            prompt: Some(prompt),
            height,
            width,
            inference: Some(inference),
            seed,
            ..Default::default()
        }
    }
}

impl Parallel {
    pub async fn run(args: Parallel) -> Result<(), &'static str> {
        assert!(args.inference % 8 == 0);

        let gpus = tch::Cuda::device_count();
        let dims = tch::Cuda::device_count() + 1;
        tch::manual_seed(args.seed);
        let init = tch::Tensor::randn(
            [dims, 4, args.height / 8, args.width / 8],
            (tch::Kind::Float, tch::Device::Cpu),
        );

        let multi_progress = MultiProgress::new();
        let progress_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-");

        let interval_size = args.inference / gpus;

        let prompt = args.prompt.clone();
        let tokenizer = Tokenizer::create(Default::default()).expect("cannot create tokenizer");
        let text = Clip::new(None)
            .and_then(|clip| clip.run(&prompt, tokenizer))
            .ok()
            .unwrap();

        let global_config = PipeConfig::from(args.clone());

        let handles = gpus!()
            .map(|gpu| {
                let device = device!(gpu);

                let config = global_config.with_device(device);
                let config = config.with_frame((interval_size as u32) * (gpu as u32));

                let init1 = tch::Tensor::copy(&init.i(gpu as i64).unsqueeze(0)).to_device(device);
                let init2 =
                    tch::Tensor::copy(&init.i((gpu + 1) as i64).unsqueeze(0)).to_device(device);
                let text = tch::Tensor::copy(&text).to_device(device);

                let progress_bar =
                    multi_progress.add(ProgressBar::new(interval_size.try_into().unwrap()));
                progress_bar.set_style(progress_style.clone());

                let multi_progress_clone = multi_progress.clone();

                tokio::spawn(async move {
                    let no_grad_guard = tch::no_grad_guard();
                    tch::Cuda::manual_seed(args.seed.try_into().unwrap());

                    multi_progress_clone.println("Build the pipe!").unwrap();

                    let mut pipe = Pipe::new(config).ok().unwrap();
                    for (k, x) in linspace(0, 1, interval_size) {
                        progress_bar.set_message(format!(
                            "Images #{}",
                            (interval_size as u64) * (gpu as u64) + k + 1
                        ));
                        slerp(x, &init1, &init2)
                            .and_then(|latents| pipe.diffuse(&latents, &text, false, None))
                            .ok()
                            .unwrap();
                        progress_bar.inc(1);
                    }

                    multi_progress_clone
                        .println(format!("Batch {device:?} is done!"))
                        .unwrap();
                    progress_bar.finish_with_message("generation");

                    drop(no_grad_guard);
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.await.unwrap();
        }

        multi_progress.clear().unwrap();

        Ok(())
    }
}
