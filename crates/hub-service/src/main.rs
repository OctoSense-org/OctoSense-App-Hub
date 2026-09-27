#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::var("OCTOSENSE_HUB_DB").map_err(|_| "OCTOSENSE_HUB_DB is required")?;
    let bind = std::env::var("OCTOSENSE_HUB_BIND").unwrap_or_else(|_| "127.0.0.1:8790".into());
    let mut service = octosense_hub_service::Service::open(std::path::Path::new(&path))?;
    if let Ok(client_id) = std::env::var("OCTOSENSE_GITHUB_CLIENT_ID") {
        let clock: std::sync::Arc<dyn octosense_hub_service::auth::Clock> =
            std::sync::Arc::new(octosense_hub_service::auth::SystemClock);
        let provider = octosense_hub_service::auth::github::GitHubDeviceProvider::new(
            &client_id,
            clock.clone(),
        )?;
        service = service.with_auth(std::sync::Arc::new(provider), clock);
    }
    let service = std::sync::Arc::new(service);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    axum::serve(listener, octosense_hub_service::router(service)).await?;
    Ok(())
}
