use tch::{
    nn,
    nn::{Module, VarStore},
    Device, Kind, Tensor,
};

use crate::diffusion::configuration::{path, Engine};

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Activation {
    QuickGelu,
    Gelu,
}

impl Module for Activation {
    fn forward(&self, xs: &Tensor) -> Tensor {
        match self {
            Activation::QuickGelu => xs * (xs * 1.702).sigmoid(),
            Activation::Gelu => xs.gelu("none"),
        }
    }
}

#[derive(Debug)]
struct ClipTextEmbeddings {
    token_embedding: nn::Embedding,
    position_embedding: nn::Embedding,
    position_ids: Tensor,
}

impl ClipTextEmbeddings {
    fn new(vs: nn::Path, c: &LMTSConfig) -> Self {
        let token_embedding = nn::embedding(
            &vs / "token_embedding",
            c.vocab_size,
            c.embed_dim,
            Default::default(),
        );
        let position_embedding = nn::embedding(
            &vs / "position_embedding",
            c.max_position_embeddings as i64,
            c.embed_dim,
            Default::default(),
        );
        let position_ids =
            Tensor::arange(c.max_position_embeddings as i64, (Kind::Int64, vs.device()))
                .expand([1, -1], false);
        ClipTextEmbeddings {
            token_embedding,
            position_embedding,
            position_ids,
        }
    }
}

impl Module for ClipTextEmbeddings {
    fn forward(&self, xs: &Tensor) -> Tensor {
        let token_embedding = self.token_embedding.forward(xs);
        let position_embedding = self.position_embedding.forward(&self.position_ids);
        token_embedding + position_embedding
    }
}

#[derive(Debug)]
struct ClipAttention {
    k_proj: nn::Linear,
    v_proj: nn::Linear,
    q_proj: nn::Linear,
    out_proj: nn::Linear,
    head_dim: i64,
    scale: f64,
    num_attention_heads: i64,
}

impl ClipAttention {
    fn new(vs: nn::Path, c: &LMTSConfig) -> Self {
        let embed_dim = c.embed_dim;
        let num_attention_heads = c.num_attention_heads;
        let k_proj = nn::linear(&vs / "k_proj", embed_dim, embed_dim, Default::default());
        let v_proj = nn::linear(&vs / "v_proj", embed_dim, embed_dim, Default::default());
        let q_proj = nn::linear(&vs / "q_proj", embed_dim, embed_dim, Default::default());
        let out_proj = nn::linear(&vs / "out_proj", embed_dim, embed_dim, Default::default());
        let head_dim = embed_dim / num_attention_heads;
        let scale = (head_dim as f64).powf(-0.5);
        ClipAttention {
            k_proj,
            v_proj,
            q_proj,
            out_proj,
            head_dim,
            scale,
            num_attention_heads,
        }
    }

    fn shape(&self, xs: &Tensor, seq_len: i64, bsz: i64) -> Tensor {
        xs.view((bsz, seq_len, self.num_attention_heads, self.head_dim))
            .transpose(1, 2)
            .contiguous()
    }

    fn forward(&self, xs: &Tensor, causal_attention_mask: &Tensor) -> Tensor {
        let (bsz, tgt_len, embed_dim) = xs.size3().unwrap();
        let query_states = xs.apply(&self.q_proj) * self.scale;
        let proj_shape = (bsz * self.num_attention_heads, -1, self.head_dim);
        let query_states = self.shape(&query_states, tgt_len, bsz).view(proj_shape);
        let key_states = self
            .shape(&xs.apply(&self.k_proj), -1, bsz)
            .view(proj_shape);
        let value_states = self
            .shape(&xs.apply(&self.v_proj), -1, bsz)
            .view(proj_shape);
        let attn_weights = query_states.bmm(&key_states.transpose(1, 2));

        let src_len = key_states.size()[1];
        let attn_weights = attn_weights.view((bsz, self.num_attention_heads, tgt_len, src_len))
            + causal_attention_mask;
        let attn_weights = attn_weights.view((bsz * self.num_attention_heads, tgt_len, src_len));
        let attn_weights = attn_weights.softmax(-1, Kind::Float);

        let attn_output = attn_weights.bmm(&value_states);
        attn_output
            .view((bsz, self.num_attention_heads, tgt_len, self.head_dim))
            .transpose(1, 2)
            .reshape(&[bsz, tgt_len, embed_dim])
            .apply(&self.out_proj)
    }
}

#[derive(Debug)]
struct ClipMlp {
    fc1: nn::Linear,
    fc2: nn::Linear,
    activation: Activation,
}

impl ClipMlp {
    fn new(vs: nn::Path, c: &LMTSConfig) -> Self {
        let fc1 = nn::linear(
            &vs / "fc1",
            c.embed_dim,
            c.intermediate_size,
            Default::default(),
        );
        let fc2 = nn::linear(
            &vs / "fc2",
            c.intermediate_size,
            c.embed_dim,
            Default::default(),
        );
        ClipMlp {
            fc1,
            fc2,
            activation: c.activation,
        }
    }
}

impl Module for ClipMlp {
    fn forward(&self, xs: &Tensor) -> Tensor {
        let xs = xs.apply(&self.fc1);
        self.activation.forward(&xs).apply(&self.fc2)
    }
}

#[derive(Debug)]
struct ClipEncoderLayer {
    self_attn: ClipAttention,
    layer_norm1: nn::LayerNorm,
    mlp: ClipMlp,
    layer_norm2: nn::LayerNorm,
}

impl ClipEncoderLayer {
    fn new(vs: nn::Path, c: &LMTSConfig) -> Self {
        let self_attn = ClipAttention::new(&vs / "self_attn", c);
        let layer_norm1 =
            nn::layer_norm(&vs / "layer_norm1", vec![c.embed_dim], Default::default());
        let mlp = ClipMlp::new(&vs / "mlp", c);
        let layer_norm2 =
            nn::layer_norm(&vs / "layer_norm2", vec![c.embed_dim], Default::default());
        ClipEncoderLayer {
            self_attn,
            layer_norm1,
            mlp,
            layer_norm2,
        }
    }

    fn forward(&self, xs: &Tensor, causal_attention_mask: &Tensor) -> Tensor {
        let residual = xs;
        let xs = self.layer_norm1.forward(xs);
        let xs = self.self_attn.forward(&xs, causal_attention_mask);
        let xs = xs + residual;

        let residual = &xs;
        let xs = self.layer_norm2.forward(&xs);
        let xs = self.mlp.forward(&xs);
        xs + residual
    }
}

#[derive(Debug)]
struct ClipEncoder {
    layers: Vec<ClipEncoderLayer>,
}

impl ClipEncoder {
    fn new(vs: nn::Path, c: &LMTSConfig) -> Self {
        let vs = &vs / "layers";
        let mut layers: Vec<ClipEncoderLayer> = Vec::new();
        for index in 0..c.num_hidden_layers {
            let layer = ClipEncoderLayer::new(&vs / index, c);
            layers.push(layer)
        }
        ClipEncoder { layers }
    }

    fn forward(&self, xs: &Tensor, causal_attention_mask: &Tensor) -> Tensor {
        let mut xs = xs.shallow_clone();
        for layer in self.layers.iter() {
            xs = layer.forward(&xs, causal_attention_mask)
        }
        xs
    }
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct LMTSConfig {
    pub vocab_size: i64,
    pub embed_dim: i64,         // aka config.hidden_size
    pub activation: Activation, // aka config.hidden_act
    pub intermediate_size: i64,
    pub max_position_embeddings: usize,
    pub num_hidden_layers: i64,
    pub num_attention_heads: i64,
    #[allow(dead_code)]
    pub projection_dim: i64,
}

impl Default for LMTSConfig {
    fn default() -> Self {
        Self {
            vocab_size: 49408,
            embed_dim: 1024,
            intermediate_size: 4096,
            max_position_embeddings: 77,
            num_hidden_layers: 23,
            num_attention_heads: 16,
            projection_dim: 512,
            activation: Activation::Gelu,
        }
    }
}

#[derive(Debug)]
pub struct ClipTextTransformer {
    embeddings: ClipTextEmbeddings,
    encoder: ClipEncoder,
    final_layer_norm: nn::LayerNorm,
    pub store: VarStore,
}

impl ClipTextTransformer {
    pub fn new(config: LMTSConfig, device: Device) -> Result<Self, &'static str> {
        let mut vs_clip = VarStore::new(device);

        let embeddings =
            ClipTextEmbeddings::new(&vs_clip.root() / "text_model" / "embeddings", &config);

        let encoder = ClipEncoder::new(&vs_clip.root() / "text_model" / "encoder", &config);

        let final_layer_norm = nn::layer_norm(
            &vs_clip.root() / "text_model" / "final_layer_norm",
            vec![config.embed_dim],
            Default::default(),
        );

        vs_clip
            .load(path::weights(Engine::LMTS))
            .map_err(|_| "Cannot load the weights")?;

        Ok(ClipTextTransformer {
            embeddings,
            encoder,
            final_layer_norm,
            store: vs_clip,
        })
    }

    fn build_causal_attention_mask(bsz: i64, seq_len: i64, device: Device) -> Tensor {
        let mut mask = Tensor::ones(&[bsz, seq_len, seq_len], (Kind::Float, device));
        mask.fill_(f32::MIN as f64).triu_(1).unsqueeze(1)
    }
}

impl Module for ClipTextTransformer {
    fn forward(&self, xs: &Tensor) -> Tensor {
        let (bsz, seq_len) = xs.size2().unwrap();
        let xs = self.embeddings.forward(xs);
        let causal_attention_mask = Self::build_causal_attention_mask(bsz, seq_len, xs.device());
        let xs = self.encoder.forward(&xs, &causal_attention_mask);
        xs.apply(&self.final_layer_norm)
    }
}
