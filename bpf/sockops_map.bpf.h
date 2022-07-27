/* SPDX-License-Identifier: GPL-2.0 */
#include <linux/bpf.h>

char _license[] SEC("license") = "GPL";

#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
#define bpf_htons(x) __builtin_bswap16(x)
#define bpf_htonl(x) __builtin_bswap32(x)
#elif __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
#define bpf_htons(x) (x)
#define bpf_htonl(x) (x)
#else
#error "__BYTE_ORDER__ error"
#endif

struct socket_key {
  __u32 src_ip;
  __u32 dst_ip;
  __u16 src_port;
  __u16 dst_port;
};

// Explicitly only add src and dst Port
struct {
  __uint(type, BPF_MAP_TYPE_SOCKHASH); 
  __type(key, struct socket_key);
  __type(value, __u32); 
  __uint(max_entries, 1);
} socket_map SEC(".maps");
