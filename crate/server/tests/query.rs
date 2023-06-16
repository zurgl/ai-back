use axum::{
    body::Body,
    http::{self, Request},
    Router,
};
use serde_json::json;
use server::app::command::playload;
use tower::ServiceExt;

use shared::constants::route;
use shared::types::CommandType;
use shared::types::ModelType;

const SENTIMENT_INPUT: &str = "Probably my all-time favorite movie, a story of selflessness, sacrifice and dedication to a noble cause, but it's not preachy or boring.";

const SUMMARZE_INPUT: [&str; 1] = ["In findings published Tuesday in Cornell University's arXiv by a team of scientists \
from the University of Montreal and a separate report published Wednesday in Nature Astronomy by a team \
from University College London (UCL), the presence of water vapour was confirmed in the atmosphere of K2-18b, \
a planet circling a star in the constellation Leo. This is the first such discovery in a planet in its star's \
habitable zone — not too hot and not too cold for liquid water to exist. The Montreal team, led by Björn Benneke, \
used data from the NASA's Hubble telescope to assess changes in the light coming from K2-18b's star as the planet \
passed between it and Earth. They found that certain wavelengths of light, which are usually absorbed by water, \
weakened when the planet was in the way, indicating not only does K2-18b have an atmosphere, but the atmosphere \
contains water in vapour form. The team from UCL then analyzed the Montreal team's data using their own software \
and confirmed their conclusion. This was not the first time scientists have found signs of water on an exoplanet, \
but previous discoveries were made on planets with high temperatures or other pronounced differences from Earth. \
\"This is the first potentially habitable planet where the temperature is right and where we now know there is water,\" \
said UCL astronomer Angelos Tsiaras. \"It's the best candidate for habitability right now.\" \"It's a good sign\", \
said Ryan Cloutier of the Harvard–Smithsonian Center for Astrophysics, who was not one of either study's authors. \
\"Overall,\" he continued, \"the presence of water in its atmosphere certainly improves the prospect of K2-18b being \
a potentially habitable planet, but further observations will be required to say for sure. \" \
K2-18b was first identified in 2015 by the Kepler space telescope. It is about 110 light-years from Earth and larger \
but less dense. Its star, a red dwarf, is cooler than the Sun, but the planet's orbit is much closer, such that a year \
on K2-18b lasts 33 Earth days. According to The Guardian, astronomers were optimistic that NASA's James Webb space \
telescope — scheduled for launch in 2021 — and the European Space Agency's 2028 ARIEL program, could reveal more \
about exoplanets like K2-18b."];

pub const TRANSLATION_INPUT: [&str; 1] =
    ["This sentence will be translated in multiple languages."];

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
struct Input {
    input: String,
}

impl Input {
    fn input(model_type: &ModelType) -> String {
        match model_type {
            ModelType::Sentiment => serde_json::to_string(&Input {
                input: SENTIMENT_INPUT.to_string(),
            })
            .ok()
            .unwrap(),
            ModelType::Summarize => serde_json::to_string(&Input {
                input: serde_json::to_string(&SUMMARZE_INPUT).ok().unwrap(),
            })
            .ok()
            .unwrap(),
            ModelType::Translation => serde_json::to_string(&Input {
                input: serde_json::to_string(&TRANSLATION_INPUT).ok().unwrap(),
            })
            .ok()
            .unwrap(),
            _ => unimplemented!(),
        }
    }
}

#[allow(dead_code)]
fn owner_from_cookie(cookie: &str) -> &str {
    let (user_id, _) = cookie.split_once(';').unwrap();
    let (_, user_id) = user_id.split_once('=').unwrap();
    user_id
}

pub async fn get_cookie(app: &Router) -> String {
    let app = app.clone();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(route::COOKIE_URL)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let header_value = response.headers().get("set-cookie").unwrap();
    header_value.to_str().ok().unwrap().to_string()
}

async fn new_req(cookie: &str, route: &str, body: Body) -> Request<Body> {
    Request::builder()
        .method(http::Method::POST)
        .uri(route)
        .header("Cookie", cookie)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}

pub async fn post_cmd(
    app: &Router,
    cookie: &str,
    model_type: ModelType,
    cmd: CommandType,
    task_id: Option<&str>,
) {
    let (payload, route) = match cmd {
        CommandType::Spawn => (
            serde_json::to_vec(&json!(playload::Spawn::new(model_type))),
            route::API_COMMAND_SPAWN_URL,
        ),
        CommandType::Process => (
            serde_json::to_vec(&json!(playload::Process::new(
                model_type,
                task_id.unwrap(),
                &Input::input(&model_type)
            ))),
            route::API_COMMAND_PROCESS_URL,
        ),
        CommandType::Kill => (
            serde_json::to_vec(&json!(playload::Kill::new(model_type, task_id.unwrap()))),
            route::API_COMMAND_KILL_URL,
        ),
        CommandType::Pause => (
            serde_json::to_vec(&json!(playload::Pause::new(model_type, task_id.unwrap()))),
            route::API_COMMAND_PAUSE_URL,
        ),
        CommandType::Resume => (
            serde_json::to_vec(&json!(playload::Resume::new(model_type, task_id.unwrap()))),
            route::API_COMMAND_RESUME_URL,
        ),
    };
    let body = Body::from(payload.ok().unwrap());
    let req = new_req(cookie, route, body).await;
    let reponse = app.clone().oneshot(req).await.unwrap();
    assert_eq!(reponse.status(), http::StatusCode::CREATED);
}
