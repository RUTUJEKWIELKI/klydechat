use std::net::{Ipv4Addr, SocketAddr};

use axum::Router;

use tokio::net::TcpListener;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

mod api;
pub mod requests;
pub mod site_models;
pub mod specialty;
pub mod website_embed;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Configure logging and environment
    revolt_config::configure!(proxy);

    // Configure API schema
    #[derive(OpenApi)]
    #[openapi(
        paths(api::root, api::proxy, api::embed),
        components(schemas(
            api::RootResponse,
            revolt_result::Error,
            revolt_result::ErrorType,
            revolt_models::v0::ImageSize,
            revolt_models::v0::Image,
            revolt_models::v0::Video,
            revolt_models::v0::TwitchType,
            revolt_models::v0::LightspeedType,
            revolt_models::v0::BandcampType,
            revolt_models::v0::Special,
            revolt_models::v0::WebsiteMetadata,
            revolt_models::v0::Text,
            revolt_models::v0::Embed
        ))
    )]
    struct ApiDoc;

    #[allow(dead_code)]
    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
                )
            }
        }
    }

    // Configure Axum and router
    let app = Router::new()
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .nest("/", api::router().await);

    // Serve the API both at its public path prefix and at the root, so that
    // public traffic (routed with the prefix intact) and internal service
    // binding calls (which use the bare root) both resolve.
    let app = Router::new().nest("/january", app.clone()).merge(app);

    // Configure TCP listener and bind
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(14705);
    tracing::info!("Listening on 0.0.0.0:{port}");
    tracing::info!("Play around with the API: http://localhost:{port}/scalar");
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = TcpListener::bind(&address).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
