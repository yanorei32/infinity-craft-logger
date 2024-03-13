use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Mutex;

use actix_web::{
    get, http::header::ContentType, post, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use clap::Parser;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static RECIPES: OnceCell<Mutex<Vec<RecipeRecord>>> = OnceCell::new();
static TOKENS: OnceCell<Vec<String>> = OnceCell::new();
static HTTPS: OnceCell<bool> = OnceCell::new();
static DB_PATH: OnceCell<PathBuf> = OnceCell::new();

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, env)]
    db_path: Option<PathBuf>,

    #[clap(long, env)]
    tokens: Vec<String>,

    #[clap(long, env)]
    https: bool,

    #[clap(long, env)]
    #[clap(default_value = "127.0.0.1:8888")]
    listen: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Recipe {
    first: String,
    second: String,
    result: String,
    emoji: String,
}

impl From<(Recipe, Option<String>)> for RecipeRecord {
    fn from(value: (Recipe, Option<String>)) -> Self {
        Self {
            first: value.0.first,
            second: value.0.second,
            result: value.0.result,
            emoji: value.0.emoji,
            client_token: value.1,
        }
    }
}

impl From<RecipeRecord> for Recipe {
    fn from(value: RecipeRecord) -> Self {
        Self {
            first: value.first,
            second: value.second,
            result: value.result,
            emoji: value.emoji,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RecipeRecord {
    first: String,
    second: String,
    result: String,
    emoji: String,
    client_token: Option<String>,
}

#[derive(Deserialize)]
struct ClientScriptQuery {
    token: String,
}

#[get("/api/infinite-craft/client-script.user.js")]
async fn api_clientscript(
    query: web::Query<ClientScriptQuery>,
    req: HttpRequest,
) -> impl Responder {
    let client_script = include_str!("../assets/client-script.user.js")
        .replace(
            "HOST",
            &req.headers().get("host").unwrap().to_str().unwrap(),
        )
        .replace("TOKEN", &query.token)
        .replace("AUTH", &(!TOKENS.get().unwrap().is_empty()).to_string())
        .replace(
            "SCHEME",
            match *HTTPS.get().unwrap() {
                true => "https",
                false => "http",
            },
        );

    HttpResponse::Ok()
        .content_type(ContentType::octet_stream())
        .body(client_script)
}

fn permitted(req: &HttpRequest) -> Result<Option<String>, ()> {
    let tokens = TOKENS.get().unwrap();

    if tokens.is_empty() {
        return Ok(None);
    };

    let Some(Ok(auth_header)) = req.headers().get("Authorization").map(|h| h.to_str()) else {
        return Err(());
    };

    for token in tokens {
        if auth_header == format!("Bearer {token}") {
            return Ok(Some(token.to_owned()));
        }
    }

    Err(())
}

#[get("/api/infinite-craft/recipes")]
async fn api_recipes(req: HttpRequest) -> impl Responder {
    if permitted(&req).is_err() {
        return HttpResponse::Forbidden().body("");
    }

    let recipes: Vec<Recipe> = RECIPES
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .map(|r| r.into())
        .collect();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&recipes.clone()).unwrap())
}

#[post("/api/infinite-craft/recipe")]
async fn api_recipe(recipe: web::Json<Recipe>, req: HttpRequest) -> impl Responder {
    let Ok(token) = permitted(&req) else {
        return HttpResponse::Forbidden().body("");
    };

    let mut recipes = RECIPES.get().unwrap().lock().unwrap();

    if recipes
        .iter()
        .any(|v| v.first == recipe.first && v.second == recipe.second)
    {
        println!("Known: {:?}", recipe.0);
        return HttpResponse::Ok().body("Ok");
    }

    println!("Added: {:?}", recipe.0);

    recipes.push((recipe.0, token).into());

    let recipes = recipes.clone();

    let mut f = File::create(DB_PATH.get().unwrap()).unwrap();
    f.write_all(serde_json::to_string(&recipes).unwrap().as_bytes())
        .unwrap();

    HttpResponse::Created().body("Added")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let c = Cli::parse();

    let db_path = match c.db_path {
        Some(v) => v,
        None => {
            let mut p = dirs::config_dir().unwrap();
            p.push("infinite_craft_recipes.json");
            p
        }
    };

    RECIPES
        .set(Mutex::new(if db_path.exists() {
            let mut recipes = File::open(&db_path).unwrap();
            let mut s = String::new();
            recipes.read_to_string(&mut s).unwrap();
            serde_json::from_str(&s).unwrap()
        } else {
            Vec::new()
        }))
        .unwrap();

    DB_PATH.set(db_path).unwrap();
    HTTPS.set(c.https).unwrap();
    TOKENS.set(c.tokens).unwrap();

    HttpServer::new(|| {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(web::redirect(
                "/",
                "https://github.com/yanorei32/infinite-craft-logger",
            ))
            .service(api_recipe)
            .service(api_recipes)
            .service(api_clientscript)
    })
    .bind(c.listen)?
    .run()
    .await
}
