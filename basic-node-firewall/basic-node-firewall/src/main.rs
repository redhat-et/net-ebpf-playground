use aya::{include_bytes_aligned, Bpf};
use anyhow::Context;
use aya::programs::{Xdp, XdpFlags};
use aya::maps::{perf::AsyncPerfEventArray, HashMap};
use aya::util::online_cpus;
use bytes::BytesMut;
use std::net::{self, Ipv4Addr};
use clap::Parser;
use tokio::{signal, task};

use basic_node_firewall_common::{PacketLog, PacketFiveTuple};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "virbr0")]
    iface: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/basic-node-firewall"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/basic-node-firewall"
    ))?;
    let program: &mut Xdp = bpf.program_mut("basic_node_firewall").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    // (1)
    let mut blocklist: HashMap<_, PacketFiveTuple, u32> =
        HashMap::try_from(bpf.map_mut("BLOCKLIST")?)?;

    // (2)
    let firewall_key = PacketFiveTuple {
        src_address: Ipv4Addr::new(192, 168, 122, 1).try_into()?, 
        dst_address: Ipv4Addr::new(192, 168, 122, 91).try_into()?,
        src_port: 61235, 
        dst_port: 8000, 
        protocol: 6,
        _pad: [0, 0, 0],
    };

    // (3)
    blocklist.insert(firewall_key, 0, 0)?;

    let mut perf_array = AsyncPerfEventArray::try_from(bpf.map_mut("EVENTS")?)?;

    for cpu_id in online_cpus()? {
        let mut buf = perf_array.open(cpu_id, None)?;

        task::spawn(async move {
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(1024))
                .collect::<Vec<_>>();

            loop {
                let events = buf.read_events(&mut buffers).await.unwrap();
                for i in 0..events.read {
                    let buf = &mut buffers[i];
                    let ptr = buf.as_ptr() as *const PacketLog;
                    let data = unsafe { ptr.read_unaligned() };
                    let src_addr = net::Ipv4Addr::from(data.ipv4_address);
                    println!("LOG: SRC {}, ACTION {}", src_addr, data.action);
                }
            }
        });
    }
    signal::ctrl_c().await.expect("failed to listen for event");
    Ok::<_, anyhow::Error>(())
}
