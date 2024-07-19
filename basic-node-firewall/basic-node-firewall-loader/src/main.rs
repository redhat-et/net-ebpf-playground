use aya::{BpfLoader, include_bytes_aligned};
use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};
use clap::Parser;

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "virbr0")]
    iface: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    env_logger::init();

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = BpfLoader::new()
        // load pinned maps from /sys/fs/bpf/my-program
        .map_pin_path("/sys/fs/bpf/basic-node-firewall")
        // finally load the code
        .load(include_bytes_aligned!("../../target/bpfel-unknown-none/debug/basic-node-firewall-loader"))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = BpfLoader::new()
        // load pinned maps from /sys/fs/bpf/my-program
        .map_pin_path("/sys/fs/bpf/basic-node-firewall")
        // finally load the code
        .load(include_bytes_aligned!("../../target/bpfel-unknown-none/release/basic-node-firewall-loader"))?;


    let program: &mut Xdp = bpf.program_mut("basic_node_firewall").unwrap().try_into()?;
    
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;
  Ok(())
}
