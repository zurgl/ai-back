use clap::{Parser, Subcommand};

use models::llama::Llama;
use models::sentiment::Sentiment;
use models::summarize::Summarize;
use models::translation::Translation;

#[derive(Debug, Clone, Subcommand)]
pub enum LoaderCmd {
    List,
    Clean,
    All,
    Remove {
        model: String,
        ressource: Option<String>,
    },
    Add {
        model: String,
        ressource: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum DiffusionCmd {
    Oneshot {
        #[arg(
            long,
            default_value = "star physic cosmos dust twister world water god"
        )]
        prompt: String,
        #[arg(long, default_value_t = 768)]
        height: i64,
        #[arg(long, default_value_t = 768)]
        width: i64,
        #[arg(long, default_value_t = 2357)]
        seed: i64,
    },
    Sequence {
        #[arg(
            long,
            default_value = "painting husky at the moutain funny carton manga"
        )]
        prompt: String,
        #[arg(long, default_value_t = 768)]
        height: i64,
        #[arg(long, default_value_t = 768)]
        width: i64,
        #[arg(long)]
        inference: i64,
        #[arg(long, default_value_t = 235742)]
        seed: i64,
    },
    Parallel {
        #[arg(long, default_value = "dreams cosmic star light water heaven")]
        prompt: String,
        #[arg(long, default_value_t = 1072)]
        height: i64,
        #[arg(long, default_value_t = 1504)]
        width: i64,
        #[arg(long, default_value_t = 8)]
        inference: i64,
        #[arg(long, default_value_t = 42)]
        seed: i64,
    },
}

// #[derive(Debug, Clone, Subcommand)]
// pub enum ServerCmd {
//     Run {
//         port: Option<String>,
//         ip: Option<String>,
//     },
// }

#[derive(Debug, Clone, Subcommand)]
pub enum LlmCmd {
    Translate {
        prompt: String,
        source_lang: String,
        target_lang: String,
    },
    Analyze {
        prompt: String,
    },
    Summarize {
        prompt: String,
    },
    Generate {
        prompt: String,
        #[arg(long, default_value_t = 256)]
        sample_len: usize,
        #[arg(long, default_value_t = 1.0)]
        temperature: f64,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Loader {
        #[command(subcommand)]
        cmd: LoaderCmd,
    },
    Diffusion {
        #[command(subcommand)]
        cmd: DiffusionCmd,
    },
    Serve,
    Llm {
        #[command(subcommand)]
        cmd: LlmCmd,
    },
}
#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let args = Args::parse();
    let command = args.cmd.expect("A command need to be provided");
    match command {
        Command::Loader { cmd } => match cmd {
            LoaderCmd::Clean => loader::clean().await,
            LoaderCmd::List => loader::list().await,
            LoaderCmd::All => loader::load().await,
            LoaderCmd::Remove { model, ressource } => loader::remove(model, ressource).await,
            LoaderCmd::Add { model, ressource } => loader::add(model, ressource).await,
        },
        Command::Diffusion { cmd } => match cmd {
            DiffusionCmd::Oneshot {
                prompt,
                height,
                width,
                seed,
            } => models::Oneshot::run(models::Oneshot {
                prompt,
                height,
                width,
                seed,
            }),
            DiffusionCmd::Parallel {
                prompt,
                height,
                width,
                seed,
                inference,
            } => {
                models::Parallel::run(models::Parallel {
                    prompt,
                    height,
                    width,
                    seed,
                    inference,
                })
                .await
            }
            DiffusionCmd::Sequence {
                prompt,
                height,
                width,
                seed,
                inference,
            } => models::Sequence::run(models::Sequence {
                prompt,
                height,
                width,
                seed,
                inference,
            }),
        },
        Command::Serve => server::run().await,
        Command::Llm { cmd } => match cmd {
            LlmCmd::Translate {
                prompt,
                source_lang,
                target_lang,
            } => Translation::default().try_prediction(&prompt, &source_lang, &target_lang),
            LlmCmd::Analyze { prompt } => Sentiment::default().try_prediction(&prompt),
            LlmCmd::Summarize { prompt } => Summarize::default().try_prediction(&prompt),
            LlmCmd::Generate {
                prompt,
                sample_len,
                temperature,
            } => Llama::default().try_prediction(&prompt, sample_len, temperature, None),
        },
    }
}
