use crate::TcpIncoming;
use roa_core::{App, Server, State};
use std::future::Future;
use std::net::{SocketAddr, ToSocketAddrs};

/// An implementation of hyper::rt::Executor based on async-std
#[derive(Copy, Clone)]
pub struct Executor;

impl<F> roa_core::Executor<F> for Executor
where
    F: 'static + Send + Future,
    F::Output: 'static + Send,
{
    #[inline]
    fn execute(&self, fut: F) {
        async_std::task::spawn(fut);
    }
}

pub trait TcpServer {
    /// tcp server
    type Server;

    /// Listen on a socket addr, return a server and the real addr it binds.
    fn listen_on(
        &self,
        addr: impl ToSocketAddrs,
    ) -> std::io::Result<(SocketAddr, Self::Server)>;

    /// Listen on a socket addr, return a server, and pass real addr to the callback.
    fn listen(
        &self,
        addr: impl ToSocketAddrs,
        callback: impl Fn(SocketAddr),
    ) -> std::io::Result<Self::Server>;

    /// Listen on an unused port of 127.0.0.1, return a server and the real addr it binds.
    /// ### Example
    /// ```rust
    /// use roa_core::App;
    /// use roa_tcp::TcpServer;
    /// use roa_core::http::StatusCode;
    /// use async_std::task::spawn;
    /// use std::time::Instant;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let (addr, server) = App::new(())
    ///         .gate_fn(|_ctx, next| async move {
    ///             let inbound = Instant::now();
    ///             next.await?;
    ///             println!("time elapsed: {} ms", inbound.elapsed().as_millis());
    ///             Ok(())
    ///         })
    ///         .run_local()?;
    ///     spawn(server);
    ///     let resp = reqwest::get(&format!("http://{}", addr)).await?;
    ///     assert_eq!(StatusCode::OK, resp.status());
    ///     Ok(())
    /// }
    /// ```
    fn run_local(&self) -> std::io::Result<(SocketAddr, Self::Server)>;
}

impl<S: State> TcpServer for App<S> {
    type Server = Server<TcpIncoming, Self, Executor>;
    fn listen_on(
        &self,
        addr: impl ToSocketAddrs,
    ) -> std::io::Result<(SocketAddr, Self::Server)> {
        let incoming = TcpIncoming::bind(addr)?;
        let local_addr = incoming.local_addr();
        let server = Server::builder(incoming)
            .executor(Executor)
            .serve(self.clone());
        Ok((local_addr, server))
    }

    fn listen(
        &self,
        addr: impl ToSocketAddrs,
        callback: impl Fn(SocketAddr),
    ) -> std::io::Result<Self::Server> {
        let (addr, server) = self.listen_on(addr)?;
        callback(addr);
        Ok(server)
    }

    fn run_local(&self) -> std::io::Result<(SocketAddr, Self::Server)> {
        self.listen_on("127.0.0.1:0")
    }
}