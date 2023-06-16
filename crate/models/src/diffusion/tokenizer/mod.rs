use serde::{Deserialize, Serialize};
use shared::tools;
use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use tch::{Device, Tensor};

pub mod constants;

use constants::{BYTES_TO_UNICODE, PAT};

use super::configuration::path;

#[derive(serde::Deserialize, Debug, PartialEq, Clone)]
pub struct TokenizerConfig {
    pub max_position_embeddings: usize,
    pub pad_with: Option<String>,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        TokenizerConfig {
            max_position_embeddings: 77,
            pad_with: Some("!".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tokenizer {
    encoder: HashMap<String, usize>,
    #[allow(dead_code)]
    decoder: HashMap<usize, String>,
    bpe_ranks: HashMap<(String, String), usize>,
    start_of_text_token: usize,
    end_of_text_token: usize,
    max_position_embeddings: usize,
    pub pad_with: Option<String>,
}

impl Tokenizer {
    pub fn set_padding(&mut self, padding: Option<String>) {
        self.pad_with = padding;
    }

    pub fn create(config: TokenizerConfig) -> Result<Tokenizer, &'static str> {
        let TokenizerConfig {
            max_position_embeddings,
            pad_with,
        } = config;

        let bpe_file = tools::file_open(path::vocab()).map_err(|_| "Cannot open the vocad file")?;
        let bpe_lines: Result<Vec<String>, _> = std::io::BufReader::new(bpe_file).lines().collect();
        let bpe_lines = bpe_lines.map_err(|_| "Cannot read the vocab file")?;
        let bpe_lines: Result<Vec<_>, _> = bpe_lines[1..49152 - 256 - 2 + 1]
            .iter()
            .map(|line| {
                let vs: Vec<_> = line.split_whitespace().collect();
                if vs.len() != 2 {
                    panic!("expected two items got {} '{}'", vs.len(), line)
                }
                Ok::<_, &'static str>((vs[0].to_string(), vs[1].to_string()))
            })
            .collect();
        let bpe_lines = bpe_lines?;

        let mut vocab: Vec<String> = Vec::new();
        for (_index, elem) in BYTES_TO_UNICODE {
            vocab.push(elem.into())
        }
        for (_index, elem) in BYTES_TO_UNICODE {
            vocab.push(format!("{elem}</w>"));
        }
        for elem in bpe_lines.iter() {
            vocab.push(format!("{}{}", elem.0, elem.1))
        }

        let start_of_text_token = vocab.len();
        vocab.push("<|startoftext|>".to_string());
        let end_of_text_token = vocab.len();
        vocab.push("<|endoftext|>".to_string());

        let encoder: HashMap<_, _> = vocab.into_iter().enumerate().map(|(i, v)| (v, i)).collect();
        let decoder: HashMap<_, _> = encoder.iter().map(|(k, v)| (*v, k.clone())).collect();
        let bpe_ranks: HashMap<_, _> = bpe_lines
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();
        let tokenizer = Tokenizer {
            encoder,
            bpe_ranks,
            decoder,
            start_of_text_token,
            end_of_text_token,
            pad_with,
            max_position_embeddings,
        };
        Ok(tokenizer)
    }

    fn get_pairs(word: &[String]) -> HashSet<(String, String)> {
        let mut pairs = HashSet::new();
        for (i, v) in word.iter().enumerate() {
            if i > 0 {
                pairs.insert((word[i - 1].clone(), v.clone()));
            }
        }
        pairs
    }

    fn bpe(&self, token: &str) -> Vec<usize> {
        let mut word: Vec<String> = token.chars().map(|x| x.to_string()).collect();
        if word.is_empty() {
            return Vec::new();
        }
        let last_index = word.len() - 1;
        word[last_index] = format!("{}</w>", word[last_index]);
        while word.len() > 1 {
            let mut current_min = None;
            let pairs = Self::get_pairs(&word);
            for p in pairs.iter() {
                match self.bpe_ranks.get(p) {
                    None => {}
                    Some(v) => {
                        let should_replace = match current_min {
                            None => true,
                            Some((current_min, _)) => v < current_min,
                        };
                        if should_replace {
                            current_min = Some((v, p))
                        }
                    }
                }
            }
            let (first, second) = match current_min {
                None => break,
                Some((_v, (first, second))) => (first, second),
            };
            let mut new_word = vec![];
            let mut index = 0;
            while index < word.len() {
                let w = &word[index];
                if index + 1 < word.len() && w == first && &word[index + 1] == second {
                    new_word.push(format!("{first}{second}"));
                    index += 2
                } else {
                    new_word.push(w.clone());
                    index += 1
                }
            }
            word = new_word
        }
        word.iter()
            .filter_map(|x| self.encoder.get(x))
            .copied()
            .collect()
    }

    pub fn encode(&self, s: &str) -> Result<Vec<usize>, &'static str> {
        let s = s.to_lowercase();
        let mut bpe_tokens: Vec<usize> = vec![self.start_of_text_token];
        let re_expr = regex::Regex::new(PAT).ok().unwrap();
        for token in re_expr.captures_iter(&s) {
            let token = token.get(0).unwrap().as_str();
            bpe_tokens.extend(self.bpe(token))
        }
        bpe_tokens.push(self.end_of_text_token);
        bpe_tokens.resize_with(
            std::cmp::min(bpe_tokens.len(), self.max_position_embeddings - 1),
            Default::default,
        );
        let pad_with = match &self.pad_with {
            None => self.end_of_text_token,
            Some(pad_with) => match self.encoder.get(pad_with) {
                None => panic!("no encoding for padding character {}", pad_with),
                Some(v) => *v,
            },
        };
        while bpe_tokens.len() < self.max_position_embeddings {
            bpe_tokens.push(pad_with)
        }
        Ok(bpe_tokens)
    }

    pub fn encode_to_tensor(&self, s: &str, device: Device) -> Result<Tensor, &'static str> {
        let tokens = self.encode(s)?;
        let tokens: Vec<i64> = tokens.into_iter().map(|x| x as i64).collect();
        let tokens = Tensor::from_slice(&tokens).view((1, -1)).to(device);
        Ok(tokens)
    }

    #[allow(dead_code)]
    pub fn decode(&self, tokens: &[usize]) -> String {
        let s: String = tokens
            .iter()
            .map(|token| self.decoder[token].as_str())
            .collect();
        s.replace("</w>", " ")
    }
}

mod empty {
    //use crate::diffusion::configuration::{get_config, Engine};
    // fn default() -> Self {
    //     let config = get_config::<TokenizerConfig>(Engine::Tokenizer)
    //         .ok()
    //         .unwrap();
    //     *config
    // }
    // impl Default for Tokenizer {
    //     fn default() -> Self {
    //         let config = TokenizerConfig::default();

    //         let TokenizerConfig {
    //             max_position_embeddings,
    //             pad_with,
    //         } = config;

    //         let bpe_file = tools::file_open(VOCAB_PATH).ok().unwrap();
    //         let bpe_lines: Result<Vec<String>, _> = std::io::BufReader::new(bpe_file).lines().collect();
    //         let bpe_lines = bpe_lines.ok().unwrap();
    //         let bpe_lines: Result<Vec<_>, _> = bpe_lines[1..49152 - 256 - 2 + 1]
    //             .iter()
    //             .map(|line| {
    //                 let vs: Vec<_> = line.split_whitespace().collect();
    //                 if vs.len() != 2 {
    //                     anyhow::bail!("expected two items got {} '{}'", vs.len(), line)
    //                 }
    //                 Ok((vs[0].to_string(), vs[1].to_string()))
    //             })
    //             .collect();
    //         let bpe_lines = bpe_lines.ok().unwrap();

    //         let mut vocab: Vec<String> = Vec::new();
    //         for (_index, elem) in BYTES_TO_UNICODE {
    //             vocab.push(elem.into())
    //         }
    //         for (_index, elem) in BYTES_TO_UNICODE {
    //             vocab.push(format!("{elem}</w>"));
    //         }
    //         for elem in bpe_lines.iter() {
    //             vocab.push(format!("{}{}", elem.0, elem.1))
    //         }

    //         let start_of_text_token = vocab.len();
    //         vocab.push("<|startoftext|>".to_string());
    //         let end_of_text_token = vocab.len();
    //         vocab.push("<|endoftext|>".to_string());

    //         let encoder: HashMap<_, _> = vocab.into_iter().enumerate().map(|(i, v)| (v, i)).collect();
    //         let decoder: HashMap<_, _> = encoder.iter().map(|(k, v)| (*v, k.clone())).collect();
    //         let bpe_ranks: HashMap<_, _> = bpe_lines
    //             .into_iter()
    //             .enumerate()
    //             .map(|(i, v)| (v, i))
    //             .collect();

    //         Tokenizer {
    //             encoder,
    //             bpe_ranks,
    //             decoder,
    //             start_of_text_token,
    //             end_of_text_token,
    //             pad_with,
    //             max_position_embeddings,
    //         }
    //     }
    // }
}
