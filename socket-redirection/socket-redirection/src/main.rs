use std::io;

use aya::{
    include_bytes_aligned,
    maps::{MapRefMut, SockHash},
    programs::{SkMsg, SockOps},
    Bpf,
};
use aya_log::BpfLogger;
use socket_redirection_common::SockKey;

use log::info;
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use thiserror::Error;
use tokio::signal;

#[derive(Error, Debug)]
pub enum Error {
    #[error("path to cgroup is not valid")]
    InvalidCgroup(#[from] io::Error),
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_target_level(LevelFilter::Error)
            .set_location_level(LevelFilter::Error)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/socket-redirection"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/socket-redirection"
    ))?;
    BpfLogger::init(&mut bpf)?;

    let sock_ops: &mut SockOps = bpf.program_mut("sockops").unwrap().try_into()?;
    sock_ops.load()?;

    let pod1_cgroup = std::fs::File::open("/sys/fs/cgroup/system.slice/runc-pod1.scope")
        .map_err(Error::InvalidCgroup)?;
    sock_ops.attach(pod1_cgroup)?;
    let pod2_cgroup = std::fs::File::open("/sys/fs/cgroup/system.slice/runc-pod2.scope")
        .map_err(Error::InvalidCgroup)?;
    sock_ops.attach(pod2_cgroup)?;
    let pod3_cgroup = std::fs::File::open("/sys/fs/cgroup/system.slice/runc-pod3.scope")
    .map_err(Error::InvalidCgroup)?;
    sock_ops.attach(pod3_cgroup)?;

    let sock_map = SockHash::<MapRefMut, SockKey>::try_from(bpf.map_mut("TCP_CONNS")?)?;

    let redir: &mut SkMsg = bpf.program_mut("socket_redirection").unwrap().try_into()?;
    redir.load()?;
    redir.attach(&sock_map)?;

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
