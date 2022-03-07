use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // web::Data wraps the connection in Arc<T> which is a smart pointer
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added with the wrap
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscription", web::post().to(subscribe))
            //     Register the connection as part of the application state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
