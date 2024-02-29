use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::sync::Mutex;

use actix_web::{
    get, http::header::ContentType, post, web, App, HttpResponse, HttpServer, Responder,
};
use clap::Parser;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

static RECIPES: OnceCell<Mutex<Vec<Recipe>>> = OnceCell::new();

#[derive(Debug, Parser)]
struct Cli {
    #[clap(default_value = "127.0.0.1:8888")]
    host: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Recipe {
    first: String,
    second: String,
    result: String,
    emoji: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("<h1>InfiniteCraftLogger</h1>")
}

#[get("/api/infinite-craft/recipes")]
async fn api_recipes() -> impl Responder {
    let recipes = RECIPES.get().unwrap().lock().unwrap();
    let recipes = recipes.clone();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&recipes).unwrap())
}

#[post("/api/infinite-craft/recipe")]
async fn api_recipe(recipe: web::Json<Recipe>) -> impl Responder {
    let mut path = dirs::config_dir().unwrap();
    path.push("infinite_craft_recipes.json");

    let mut recipes = RECIPES.get().unwrap().lock().unwrap();

    if recipes
        .iter()
        .any(|v| v.first == recipe.first && v.second == recipe.second)
    {
        println!("Known: {:?}", recipe.0);
        return HttpResponse::Ok().body("Ok");
    }

    println!("Added: {:?}", recipe.0);

    recipes.push(recipe.0);

    let recipes = recipes.clone();

    let mut f = File::create(path).unwrap();
    f.write_all(serde_json::to_string(&recipes).unwrap().as_bytes())
        .unwrap();

    HttpResponse::Created().body("Added")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut recipes = dirs::config_dir().unwrap();
    recipes.push("infinite_craft_recipes.json");

    let c = Cli::parse();

    RECIPES
        .set(Mutex::new(if recipes.exists() {
            let mut recipes = File::open(recipes).unwrap();
            let mut s = String::new();
            recipes.read_to_string(&mut s).unwrap();
            serde_json::from_str(&s).unwrap()
        } else {
            Vec::new()
        }))
        .unwrap();

    HttpServer::new(|| {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(hello)
            .service(api_recipe)
            .service(api_recipes)
    })
    .bind(c.host)?
    .run()
    .await
}
