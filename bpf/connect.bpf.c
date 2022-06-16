/* SPDX-License-Identifier: GPL-2.0 */
#include <linux/bpf.h>
#include <linux/pkt_cls.h>
#include <bpf/bpf_helpers.h>

SEC("cgroup/connect4")
int bpf_sockops_sctp_load(struct bpf_sock_addr *ctx)
{
    const char err_str[] = "Hello, world, from BPF! Saw connect() syscall\
: %x";

    bpf_trace_printk(err_str, sizeof(err_str), ctx->user_ip4);

	return 1;
}

char _license[] SEC("license") = "GPL";
