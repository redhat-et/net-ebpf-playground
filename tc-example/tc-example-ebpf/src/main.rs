#![no_std]
#![no_main]

use aya_bpf::{
    bindings::xdp_action,
    macros::xdp,
    programs::XdpContext,
};
use aya_log_ebpf::info;

#[xdp(name="tc_example")]
pub fn tc_example(ctx: XdpContext) -> u32 {
    match unsafe { try_tc_example(ctx) } {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

unsafe fn try_tc_example(ctx: XdpContext) -> Result<u32, u32> {
    info!(&ctx, "received a packet");
    Ok(xdp_action::XDP_PASS)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
