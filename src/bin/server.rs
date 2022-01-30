use std::{future::Future, net::SocketAddr};

use structopt::StructOpt;
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
};

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, default_value = "0.0.0.0:1234")]
    local: String,
    #[structopt(long)]
    remote: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let opts = Box::leak(Box::new(Opts::from_args()));

    let server = TcpListener::bind(&opts.local).await.unwrap();
    log::info!("Listening on address {:?}", server.local_addr());

    loop {
        let (stream, peer) = server.accept().await.unwrap();

        tokio::spawn(anyhow_wrapper(handle_stream(stream, peer, opts)));
    }
}

async fn handle_stream(stream: TcpStream, peer: SocketAddr, opts: &Opts) -> anyhow::Result<()> {
    log::info!("Accepted connection from {peer}");

    let mut ws = tokio_tungstenite::accept_async(stream).await?;
    let mut remote = TcpStream::connect(&opts.remote).await?;

    log::info!("Connected to remote {:?}", remote.peer_addr());

    let stats = copy_bidirectional(ws.get_mut(), &mut remote).await?;

    log::info!("Closing connection to {peer}, data copied: {stats:?}");

    Ok(())
}

async fn anyhow_wrapper(fut: impl Future<Output = anyhow::Result<()>>) {
    if let Err(err) = fut.await {
        log::error!("{err:?}");
    }
}
