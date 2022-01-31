#![feature(unix_socket_ancillary_data)]

use std::{
    io::IoSlice,
    os::unix::{
        net::{SocketAncillary, UnixStream},
        prelude::{AsRawFd, FromRawFd},
    },
};

use anyhow::{bail, Result};
use structopt::StructOpt;
use tungstenite::stream::MaybeTlsStream;

#[derive(StructOpt)]
struct Opts {
    #[structopt(long)]
    remote: String,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();

    let (remote, _response) = tungstenite::connect(&opts.remote)?;

    let stream = match remote.get_ref() {
        MaybeTlsStream::Plain(stream) => stream,
        _ => bail!("connect-to-ws only works with plain `ws://` uris"),
    };

    pass_to_parent(stream.as_raw_fd())?;

    Ok(())
}

fn pass_to_parent(fd: i32) -> Result<()> {
    let stdout_stream = unsafe { UnixStream::from_raw_fd(libc::STDOUT_FILENO) };

    let mut ancillary_buf = [0; 64];
    let mut ancillary = SocketAncillary::new(&mut ancillary_buf);
    ancillary.add_fds(&[fd]);

    stdout_stream.send_vectored_with_ancillary(&[IoSlice::new(&['!' as u8])], &mut ancillary)?;

    Ok(())
}
