mod api;
mod model;
mod repository;

use api::task::{
    get_task, submit_task, start_task, complete_task, fail_task, pause_task
};
use actix_web::{App, HttpServer, web::Data, middleware::Logger};
use repository::ddb::DDBRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let config: Config = aws_config::load_from_env().await;
    HttpServer::new(move || {
        let ddb_repo: DDBRepository = DDBRepository::init(
            String::from("task"),
            config.clone(),
        );
        let ddb_data: Data<DDBRepository> = Data::new(ddb_repo);
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(ddb_data)
            .service(get_task)
            .service(submit_task)
            .service(fail_task)
            .service(pause_task)
            .service(start_task)
            .service(submit_task)
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await
}
