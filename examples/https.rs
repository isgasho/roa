//! RUST_LOG=info Cargo run --example https,
//! then request https://127.0.0.1:8000.

use log::info;
use roa::body::DispositionType;
use roa::logger::logger;
use roa::preload::*;
use roa::tls::internal::pemfile::{certs, rsa_private_keys};
use roa::tls::{NoClientAuth, ServerConfig};
use roa::{App, Context};
use std::error::Error as StdError;
use std::fs::File;
use std::io::BufReader;

async fn serve_file(ctx: &mut Context) -> roa::Result {
    ctx.write_file("assets/welcome.html", DispositionType::Inline)
        .await
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    pretty_env_logger::init();

    let mut config = ServerConfig::new(NoClientAuth::new());
    let mut cert_file = BufReader::new(File::open("assets/cert.pem")?);
    let mut key_file = BufReader::new(File::open("assets/key.pem")?);
    let cert_chain = certs(&mut cert_file).unwrap();
    let mut keys = rsa_private_keys(&mut key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0))?;

    let app = App::new().gate(logger).end(serve_file);
    app.listen_tls("127.0.0.1:8000", config, |addr| {
        info!("Server is listening on https://localhost:{}", addr.port())
    })?
    .await?;
    Ok(())
}
