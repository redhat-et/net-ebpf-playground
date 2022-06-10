#![no_std]
#![no_main]

use aya_bpf::{
    bindings::{
        sk_action, BPF_F_INGRESS, BPF_SOCK_OPS_ACTIVE_ESTABLISHED_CB,
        BPF_SOCK_OPS_PASSIVE_ESTABLISHED_CB,
    },
    macros::{map, sk_msg, sock_ops},
    maps::SockHash,
    programs::{SkMsgContext, SockOpsContext},
};

const AF_INET: u32 = 2;

use aya_log_ebpf::debug;

use socket_redirection_common::SockKey;

#[map]
static TCP_CONNS: SockHash<SockKey> = SockHash::<SockKey>::with_max_entries(65535, 0);

#[sk_msg(name = "socket_redirection")]
pub fn socket_redirection(ctx: SkMsgContext) -> u32 {
    match try_socket_redirection(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_socket_redirection(ctx: SkMsgContext) -> Result<u32, u32> {
    if unsafe { (*ctx.msg).family } != AF_INET {
        debug!(&ctx, "not ipv4");
        return Err(sk_action::SK_PASS);
    }
    let remote_ip4 = unsafe { (*ctx.msg).remote_ip4 };
    let local_ip4 = unsafe { (*ctx.msg).local_ip4 };
    let remote_port = unsafe { (*ctx.msg).remote_port >> 16 };
    let local_port = unsafe { htonl((*ctx.msg).local_port) >> 16 };
    let mut key = SockKey {
        remote_ip4,
        local_ip4,
        remote_port,
        local_port,
    };
    let _ = TCP_CONNS.redirect_msg(&ctx, &mut key, BPF_F_INGRESS.into());
    Ok(sk_action::SK_PASS)
}

#[sock_ops]
pub fn sock_ops(ctx: SockOpsContext) -> u32 {
    match try_sock_ops(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_sock_ops(ctx: SockOpsContext) -> Result<u32, u32> {
    let local_ip4 = ctx.local_ip4();
    match ctx.op() {
        // Perform Redirection For Established TCP Connections
        BPF_SOCK_OPS_PASSIVE_ESTABLISHED_CB | BPF_SOCK_OPS_ACTIVE_ESTABLISHED_CB => {
            if ctx.family() == AF_INET {
                let remote_ip4 = local_ip4;
                let local_ip4 = ctx.remote_ip4();
                let remote_port = htonl(ctx.local_port()) >> 16;
                let local_port = ctx.remote_port() >> 16;
                let mut key = SockKey {
                    remote_ip4,
                    local_ip4,
                    remote_port,
                    local_port,
                };
                let _ = unsafe { TCP_CONNS.update(&mut key, &mut *ctx.ops, 0) };
                debug!(
                    &ctx,
                    "sock ops: remote_ip: {}, local_ip: {}, remote_port: {}, local_port: {}",
                    remote_ip4,
                    local_ip4,
                    remote_port,
                    local_port
                );
            }
        }
        _ => {}
    }
    Ok(0)
}

pub fn htonl(u: u32) -> u32 {
    u.to_be()
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
