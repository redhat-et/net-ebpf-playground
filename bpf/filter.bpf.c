/* SPDX-License-Identifier: GPL-2.0 */
#include <linux/bpf.h>
#include <linux/pkt_cls.h>
#include <bpf/bpf_helpers.h>

SEC("classifier")
int cls_main(struct __sk_buff *skb)
{
    return TC_ACT_OK;
}

char _license[] SEC("license") = "GPL";
