use std::env;

async fn main_internal() {
    dotenv::dotenv().ok();
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("info,sqlx::query=info,web3=warn".to_string()),
    );

    env_logger::init();
}

#[actix_web::main]
async fn main() {
    main_internal().await;
}
