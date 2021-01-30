// Required for database/schema.rs
#[macro_use]
extern crate diesel;

// Required for embed_migrations macro in database/mod.rs
#[macro_use]
extern crate diesel_migrations;

use actix_web::{web, App, HttpServer};
use anyhow::Result;

mod database;
mod error;

mod routes;
use routes::*;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    sodiumoxide::init().expect("Failed to init crypto library!");

    let db_pool = web::Data::new(database::connect()?);
    let captcha_manager = web::Data::new(CaptchaManager::new());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .app_data(captcha_manager.clone())
            .wrap(actix_web::middleware::Logger::new(
                "%t: %r -> %s; %b B; %D ms",
            ))
            .service(get_captcha)
            .service(register_user)
            .service(login_user)
            .service(put_user)
            .service(get_user)
            .service(get_person)
            .service(update_person)
            .service(get_persons)
            .service(delete_person)
            .service(get_ensemble)
            .service(update_ensemble)
            .service(delete_ensemble)
            .service(get_ensembles)
            .service(get_instrument)
            .service(update_instrument)
            .service(delete_instrument)
            .service(get_instruments)
            .service(get_work)
            .service(update_work)
            .service(delete_work)
            .service(get_works)
            .service(get_recording)
            .service(update_recording)
            .service(delete_recording)
            .service(get_recordings_for_work)
            .service(get_medium)
            .service(get_mediums_for_recording)
            .service(get_mediums_by_discid)
            .service(update_medium)
            .service(delete_medium)
    });

    server.bind("127.0.0.1:8087")?.run().await?;

    Ok(())
}
