use crate::error::ActorError;
use shared::constants;

use crate::worker::{diffusion, llama, sentiment, summarize, translation};
use dashmap::{mapref::multiple::RefMulti, DashMap};
use shared::types::MessageType;
use shared::types::ModelType;

use shared::{
    command::{instruction, instruction::Instruction, Command},
    message::{
        emit::{Emit, EmitSource},
        Message,
    },
    tools::root,
};
use std::{collections::hash_map::RandomState, sync::Arc};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

type TaskHandle = JoinHandle<Result<(), String>>;

#[derive(Debug)]
struct Register {
    handle: TaskHandle,
    tx: mpsc::Sender<Box<dyn Instruction>>,
    owner: String,
    model_type: ModelType,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RegisterJson {
    task_id: String,
    owner: String,
    model_type: ModelType,
}

impl From<RefMulti<'_, String, Register, RandomState>> for RegisterJson {
    fn from(value: RefMulti<String, Register, RandomState>) -> Self {
        RegisterJson {
            task_id: value.key().clone(),
            owner: value.value().owner.clone(),
            model_type: value.value().model_type,
        }
    }
}

type ModelRegister = Arc<DashMap<String, Register>>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MapJson {
    value: Vec<RegisterJson>,
}

impl From<ModelRegister> for MapJson {
    fn from(value: ModelRegister) -> Self {
        Self {
            value: value.iter().map(RegisterJson::from).collect(),
        }
    }
}

async fn model_process(
    register_map: &ModelRegister,
    instruction: instruction::Process,
) -> Result<(), ActorError> {
    let owner = instruction.owner();
    let model_type = instruction.model_type();
    let task_id = instruction.task_id();

    println!("{model_type:?}");
    println!("{task_id:?}");
    println!("{owner:?}");

    let tx = match task_id {
        Some(task_id) => {
            let pair = register_map.get(&task_id).ok_or(
                ActorError::SupervisorModelProcessTaskIdNotFound {
                    owner: instruction.owner(),
                },
            )?;
            let Register { tx, .. } = pair.value();
            tx.clone()
        }
        None => {
            let pair = register_map
                .iter()
                .find(|value| value.value().model_type == model_type)
                .unwrap();
            let Register { tx, .. } = pair.value();
            tx.clone()
        }
    };

    tx.send(Box::new(instruction))
        .await
        .map_err(|_| ActorError::SupervisorModelProcessEchoFailed { owner })
}

async fn model_kill(
    register_map: &mut ModelRegister,
    tx: &broadcast::Sender<Message>,
    instruction: instruction::Kill,
) -> Result<(), ActorError> {
    let owner = instruction.owner();
    if let Some(register) = register_map.get_mut(&instruction.task_id().unwrap()) {
        register.handle.abort();

        let boxed_instruction: Box<dyn Instruction> = Box::new(instruction);
        MessageType::ModelKilled
            .emit(tx, boxed_instruction.into(), None)
            .map_err(|_| ActorError::SupervisorModelEmitKillEmitMessageFailed { owner })
            .map(|_| Ok(()))?
    } else {
        Err(ActorError::SupervisorModelKillEmitTaskIdNotFound { owner })
    }
}

async fn health_signal(
    tx: &broadcast::Sender<Message>,
    registers: ModelRegister,
) -> Result<(), ActorError> {
    let tx = tx.clone();
    tokio::task::Builder::new()
        .name("health")
        .spawn(async move {
            loop {
                let value = serde_json::json!(MapJson::from(registers.clone()));
                if let Err(err) = MessageType::Health.emit(
                    &tx,
                    EmitSource::health(tokio::task::id()),
                    Some(&value.to_string()),
                ) {
                    println!("{err:#?}");
                }
                shared::tools::wait(constants::time::INTERVAL).await;
            }
        })
        .and(Ok(()))
        .or(Err(ActorError::SupervisorTaskStatsTaskIdNotFound {
            owner: root(),
        }))
}

async fn model_spawn(
    model_register: &mut ModelRegister,
    supervisor_tx: broadcast::Sender<Message>,
    instruction: instruction::Spawn,
) -> Result<(), ActorError> {
    let owner = instruction.owner();
    let (tx, rx) = mpsc::channel::<Box<dyn Instruction>>(constants::chan::MPSC_LEN);
    let model_count = model_register
        .iter()
        .filter(|value| value.value().model_type == instruction.model_type())
        .count();

    let model_type_clone = instruction.model_type();
    let boxed_instruction: Box<dyn Instruction> = Box::new(instruction.clone());
    if let Ok(handle) = tokio::task::Builder::new()
        .name(&format!("{model_type_clone:?}-{model_count}"))
        .spawn(async move {
            let source: EmitSource = boxed_instruction.into();
            match model_type_clone {
                ModelType::Sentiment => sentiment::run(rx, supervisor_tx, source).await,
                ModelType::Summarize => summarize::run(rx, supervisor_tx, source).await,
                ModelType::Translation => translation::run(rx, supervisor_tx, source).await,
                ModelType::Diffusion => diffusion::run(rx, supervisor_tx, source).await,
                ModelType::Llama => llama::run(rx, supervisor_tx, source).await,
            }
        })
    {
        model_register
            .insert(
                handle.id().to_string(),
                Register {
                    handle,
                    tx,
                    model_type: instruction.model_type(),
                    owner: instruction.owner,
                },
            )
            .map(|_| Err(ActorError::SupervisorModelSpawnInsertTaskIdFailed { owner }))
            .unwrap_or(Ok::<(), ActorError>(()))
    } else {
        Err(ActorError::SupervisorModelSpawnModelSpawnFailed { owner })
    }
}

pub async fn run(
    tx: broadcast::Sender<Message>,
    mut rx: mpsc::Receiver<Command>,
    models: Vec<ModelType>,
) {
    let mut model_register: ModelRegister = Arc::new(DashMap::new());

    if let Err(err) = health_signal(&tx, model_register.clone()).await {
        println!("{err}");
        std::process::exit(1);
    }

    for model in models {
        if let Err(err) = model_spawn(
            &mut model_register,
            tx.clone(),
            instruction::Spawn::new(constants::role::ROOT, model),
        )
        .await
        {
            println!("{err}");
            std::process::exit(1);
        }
    }

    while let Some(command) = rx.recv().await {
        println!("{command:?}");
        let source = EmitSource::from(command.boxed_instruction()).set_task_id(tokio::task::id());
        let response: Result<(), ActorError> = match command {
            Command::Kill(instruction) => model_kill(&mut model_register, &tx, instruction).await,
            Command::Spawn(instruction) => {
                model_spawn(&mut model_register, tx.clone(), instruction).await
            }
            Command::Process(instruction) => model_process(&model_register, instruction).await,
            _ => Err(ActorError::SupervisoRunCommandNotImplemented),
        };

        match response {
            Ok(_) => {
                if let Err(err) = MessageType::CommandSucess.emit(&tx, source.clone(), None) {
                    println!("{err:#?}");
                }
            }
            Err(error) => {
                if let Err(err) = MessageType::CommandFailed.emit(
                    &tx,
                    source.clone(),
                    Some(&format!("{error:?}")),
                ) {
                    println!("{err:#?}");
                }
            }
        };
    }
}
