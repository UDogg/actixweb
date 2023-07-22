mod api;
mod model;
mod repository;

use api::task::{
    get_task
};

use actix_web::{App, HttpServer, web::Data, middleware::Logger};
use repository::ddb::DDBRepository;

#[actix_web::main]


async fn main() -> std::io::Result<()> {
    std::env::set_var(key: "RUST_LOG", value:"debug");
    std::env::set_var(key: "RUST_BACKTRACE", value:"1");
    env_logger::init();

    let config: Config = aws_config::load_from_env().await;
    HttpServer::new(factory: move || {
        let ddb_repo: DDBRepository = DDBRepository::init(
            String::from("task"),
            config.clone(),
        );
        let ddb_data: Data<DDBRepository> = Data::new(ddb_repo);
        let logger = Logger::default();
        App::new(): App<AppEntry>
            .wrap(mw: logger): App<impl ServiceFactory<Config = (), Request = ServiceRequest, Response = ServiceResponse, Error = Error, InitError = (), Service = HttpService<Request, Response, Error, Config>>>
            .app_data(ext: ddb_data): App<impl ServiceFactory<Config = (), Request = ServiceRequest, Response = ServiceResponse, Error = Error, InitError = (), Service = HttpService<Request, Response, Error, Config>>>
            .service(factory: get_task)
    }): HttpServer<|| -> App<impl ServiceFactory<Config = (), Request = ServiceRequest, Response = ServiceResponse, Error = Error, InitError = (), Service = HttpService<Request, Response, Error, Config>>>>, Error>
    .bind(addr: ("127.0.0.1", 80))?
    .run(): Server
    .await
}
