/* SPDX-License-Identifier: GPL-2.0 */
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include "sockops_map.bpf.h"

SEC("sockops")
int load_sock(struct bpf_sock_ops *skops)
{       
    if (skops->local_port == 0x1f40) { 
        struct socket_key key = {
            .src_ip = skops->local_ip4,
            .src_port = bpf_htons(skops->local_port),
            .dst_ip = skops->remote_ip4,
            .dst_port = skops->remote_port,
        };

        // insert the source socket in the sock_ops_map
        int ret = bpf_sock_hash_update(skops, &socket_map, &key, BPF_NOEXIST);
        if (ret != 0) {
            const char err_str[] = "Failed to Load Socket\
            for VIP: %x with port: %x \n";

            bpf_trace_printk(err_str, sizeof(err_str), skops->local_ip4, skops->local_port);
        }
    }

	return 0;
}
