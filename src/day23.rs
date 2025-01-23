use std::fmt::Display;

use axum::{
    extract::{rejection::PathRejection, Path},
    http::StatusCode,
    response::Result,
    routing::{get, post},
    Router,
};
use axum_extra::extract::Multipart;
use hex::decode;
use rinja::Template;
use serde::Deserialize;
use tracing::error;

pub fn router() -> Router {
    Router::new()
        .route("/star", get(star))
        .route("/present/:color", get(present))
        .route("/ornament/:state/:n", get(ornament))
        .route("/lockfile", post(lockfile))
}

async fn star() -> Star {
    Star
}

async fn present(path: Result<Path<Color>, PathRejection>) -> Result<Present, StatusCode> {
    match path {
        Ok(Path(color)) => Ok(Present { color }),
        Err(_) => Err(StatusCode::IM_A_TEAPOT),
    }
}

async fn ornament(
    path: Result<Path<(State, String)>, PathRejection>,
) -> Result<Ornament, StatusCode> {
    match path {
        Ok(Path((state, n))) => Ok(Ornament { state, n }),
        Err(_) => Err(StatusCode::IM_A_TEAPOT),
    }
}

async fn lockfile(mut form: Multipart) -> Result<Cake, StatusCode> {
    let field = form
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let text = field
        .text()
        .await
        .inspect_err(|_| error!("failed to fetch text"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let lockfile: Lockfile = toml::from_str(&text)
        .inspect_err(|err| error!(%err, "failed to parse toml"))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let layers: Vec<Layer> = lockfile
        .packages
        .into_iter()
        .filter_map(|package| {
            let checksum = package.checksum?;
            let Ok(bytes) = decode(checksum) else {
                error!("Invalid checksum");
                return Some(Err(StatusCode::UNPROCESSABLE_ENTITY));
            };
            if bytes.len() < 5 {
                error!("Invalid checksum length");
                return Some(Err(StatusCode::UNPROCESSABLE_ENTITY));
            }
            let color = format!("#{:02x}{:02x}{:02x}", bytes[0], bytes[1], bytes[2]);
            let top = bytes[3];
            let left = bytes[4];
            Some(Ok(Layer { color, top, left }))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Cake { layers })
}

#[derive(Template)]
#[template(path = "star.html")]
struct Star;

#[derive(Template)]
#[template(path = "present.html")]
struct Present {
    color: Color,
}

impl Present {
    fn next_color(&self) -> Color {
        match self.color {
            Color::Red => Color::Blue,
            Color::Blue => Color::Purple,
            Color::Purple => Color::Red,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Color {
    Red,
    Blue,
    Purple,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Color::Red => "red",
            Color::Blue => "blue",
            Color::Purple => "purple",
        };
        write!(f, "{}", s)
    }
}

#[derive(Template)]
#[template(path = "ornament.html")]
struct Ornament {
    state: State,
    n: String,
}

impl Ornament {
    fn classes(&self) -> &str {
        match self.state {
            State::On => "ornament on",
            State::Off => "ornament",
        }
    }

    fn next_state(&self) -> State {
        match self.state {
            State::On => State::Off,
            State::Off => State::On,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum State {
    On,
    Off,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            State::On => "on",
            State::Off => "off",
        };
        write!(f, "{}", s)
    }
}

#[derive(Template)]
#[template(path = "cake.html")]
struct Cake {
    layers: Vec<Layer>,
}

struct Layer {
    color: String,
    top: u8,
    left: u8,
}

#[derive(Deserialize)]
struct Lockfile {
    #[serde(rename = "package")]
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    checksum: Option<String>,
}
