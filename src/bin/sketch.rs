use clap::Parser;
use diffusers::pipelines::stable_diffusion;
use diffusers::transformers::clip;
use tch::{nn::Module, Device, Kind, Tensor};

// cargo run --example stable-diffusion-img2img --features clap -- --input-image media/in_img2img.jpg

const GUIDANCE_SCALE: f64 = 7.5;
// const INPUT_IMAGE: &str = "images/input/in_img2img.jpg";
const INPUT_IMAGE: &str = "images/input/ez.png";
const VOCAB_FILE: &str = ".ai-data/stable_diffusion_2_1/vocab.txt";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    // #[arg(long, default_value = "A fantasy landscape")]
    #[arg(
        long,
        default_value = "A realistic portrait of a beautifull lady, colorfull, young"
    )]
    prompt: String,

    /// The number of steps to run the diffusion for.
    #[arg(long, default_value_t = 30)]
    n_steps: usize,

    /// The strength, indicates how much to transform the initial image. The
    /// value must be between 0 and 1, a value of 1 discards the initial image
    /// information.
    #[arg(long, default_value_t = 0.85)]
    strength: f64,

    #[arg(long, default_value_t = 32)]
    seed: i64,

    /// The number of samples to generate.
    #[arg(long, default_value_t = 5)]
    num_samples: i64,

    /// The name of the final image to generate.
    #[arg(long, value_name = "FILE", default_value = "final.png")]
    final_image: String,
}

impl Args {
    fn clip_weights(&self) -> String {
        ".ai-data/stable_diffusion_2_1/lmts.safetensors".to_string()
    }

    fn vae_weights(&self) -> String {
        ".ai-data/stable_diffusion_2_1/vae.safetensors".to_string()
    }

    fn unet_weights(&self) -> String {
        ".ai-data/stable_diffusion_2_1/unet.safetensors".to_string()
    }
}

fn image_preprocess<T: AsRef<std::path::Path>>(path: T) -> anyhow::Result<Tensor> {
    let image = tch::vision::image::load(path)?;
    let (_num_channels, height, width) = image.size3()?;
    let height = height - height % 32;
    let width = width - width % 32;
    let image = tch::vision::image::resize(&image, width, height)?;
    Ok((image / 255. * 2. - 1.).unsqueeze(0))
}

fn run(args: Args) -> anyhow::Result<()> {
    let clip_weights = args.clip_weights();
    let vae_weights = args.vae_weights();
    let unet_weights = args.unet_weights();
    let Args {
        prompt,
        n_steps,
        seed,
        final_image,
        num_samples,
        strength,
        ..
    } = args;
    if !(0. ..=1.).contains(&strength) {
        anyhow::bail!("strength should be between 0 and 1, got {strength}")
    }
    tch::maybe_init_cuda();
    println!("Cuda available: {}", tch::Cuda::is_available());
    println!("Cudnn available: {}", tch::Cuda::cudnn_is_available());
    let sd_config = stable_diffusion::StableDiffusionConfig::v2_1(None, None, None);

    let init_image = image_preprocess(INPUT_IMAGE)?;
    let clip_device = tch::Device::Cuda(0);
    let vae_device = tch::Device::Cuda(0);
    let unet_device = tch::Device::Cuda(0);
    let scheduler = sd_config.build_scheduler(n_steps);

    let tokenizer = clip::Tokenizer::create(VOCAB_FILE, &sd_config.clip)?;
    println!("Running with prompt \"{prompt}\".");
    let tokens = tokenizer.encode(&prompt)?;
    let tokens: Vec<i64> = tokens.into_iter().map(|x| x as i64).collect();
    let tokens = Tensor::from_slice(&tokens).view((1, -1)).to(clip_device);
    let uncond_tokens = tokenizer.encode("")?;
    let uncond_tokens: Vec<i64> = uncond_tokens.into_iter().map(|x| x as i64).collect();
    let uncond_tokens = Tensor::from_slice(&uncond_tokens)
        .view((1, -1))
        .to(clip_device);

    let no_grad_guard = tch::no_grad_guard();

    println!("Building the Clip transformer.");
    let text_model = sd_config.build_clip_transformer(&clip_weights, clip_device)?;
    let text_embeddings = text_model.forward(&tokens);
    let uncond_embeddings = text_model.forward(&uncond_tokens);
    let text_embeddings = Tensor::cat(&[uncond_embeddings, text_embeddings], 0).to(unet_device);

    println!("Building the autoencoder.");
    let vae = sd_config.build_vae(&vae_weights, vae_device)?;
    println!("Building the unet.");
    let unet = sd_config.build_unet(&unet_weights, unet_device, 4)?;

    println!(
        "Generating the latent from the input image {:?}.",
        init_image.size()
    );
    let init_image = init_image.to(vae_device);
    let init_latent_dist = vae.encode(&init_image);

    let t_start = n_steps - (n_steps as f64 * strength) as usize;

    for idx in 0..num_samples {
        tch::manual_seed(seed + idx);
        let latents = (init_latent_dist.sample() * 0.18215).to(unet_device);
        let timesteps = scheduler.timesteps();
        let noise = latents.randn_like();
        let mut latents = scheduler.add_noise(&latents, noise, timesteps[t_start]);

        for (timestep_index, &timestep) in timesteps.iter().enumerate() {
            if timestep_index < t_start {
                continue;
            }
            println!("Timestep {timestep_index}/{n_steps}");
            let latent_model_input = Tensor::cat(&[&latents, &latents], 0);
            let latent_model_input = scheduler.scale_model_input(latent_model_input, timestep);

            let noise_pred = unet.forward(&latent_model_input, timestep as f64, &text_embeddings);
            let noise_pred = noise_pred.chunk(2, 0);
            let (noise_pred_uncond, noise_pred_text) = (&noise_pred[0], &noise_pred[1]);
            let noise_pred =
                noise_pred_uncond + (noise_pred_text - noise_pred_uncond) * GUIDANCE_SCALE;
            latents = scheduler.step(&noise_pred, timestep, &latents);
        }

        println!(
            "Generating the final image for sample {}/{}.",
            idx + 1,
            num_samples
        );
        let latents = latents.to(vae_device);
        let image = vae.decode(&(&latents / 0.18215));
        let image = (image / 2 + 0.5).clamp(0., 1.).to_device(Device::Cpu);
        let image = (image * 255.).to_kind(Kind::Uint8);
        let final_image = if num_samples > 1 {
            match final_image.rsplit_once('.') {
                None => format!("{}.{}.png", final_image, idx + 1),
                Some((filename_no_extension, extension)) => {
                    format!("{}.{}.{}", filename_no_extension, idx + 1, extension)
                }
            }
        } else {
            final_image.clone()
        };
        tch::vision::image::save(&image, final_image)?;
    }

    drop(no_grad_guard);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args)
}
