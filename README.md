Experiments with eBPF for K8s
=============================

This is a work in progress

# Prerequisites

A Fedora 36 Workstation with:

1. `dnf install elfutils-libelf-devel`
2. LLVM and Clang
3. `skopeo`, `umoci` and `runc`

# Topology

Set up a test topology - where eth0 is your internet connected interface

```sh
sudo ./topology.sh up eth0
```

Tear down the topology
```sh
sudo ./topology.sh down
```

Clear the image cache
```sh
sudo ./topoligy.sh clean
```

Enter a container for testing
```sh
sudo runc exec -t pod2 /bin/sh
```

Add tools to a container
```sh
apk add --no-cache iperf
```

# Build Programs

```sh
make
```

# Attach a program

XDP

```sh
sudo ip link set pod1 xdp object ./.output/pass.bpf.o section xdp/pass
```

TC

```sh
sudo tc qdisc add dev pod1 clsact
sudo tc filter add dev pod1 ingress prio 1 handle 1 bpf object-file .output/filter.bpf.o direct-action 
```

# Run the Socket Redirection Example

```sh
cd socket-redirection
cargo xtask run
```

# Run the SK_LOOKUP Socket Redirection Example

1. Load sockops program and pin map 

```
    sudo bpftool prog -d load ./.output/sock_ops.bpf.o /sys/fs/bpf/load_sock type sockops
```

```
    sudo bpftool map pin name socket_map "/sys/fs/bpf/socket_map"
```

2. Load sklookup program 

```
    sudo bpftool prog load ./.output/sk_lookup.bpf.o /sys/fs/bpf/sk_lookup type sk_lookup map name socket_map pinned "/sys/fs/bpf/socket_map"
```

3. Attach sockops program to default cgroup which will add the server socket to our sockmap

```
sudo bpftool cgroup attach "/sys/fs/cgroup" sock_ops pinned "/sys/fs/bpf/load_sock"
```

4. Start server in `pod1`

```
    sudo ip netns exec pod1 python3 -m http.server
```

5. Ensure server socket is loaded into map 


``` 
    sudo bpftool map dump name socket_map
    key:
    00 00 00 00 00 00 00 00  1f 40 00 00
    value:
    No space left on device
    Found 0 elements
```


6. Attach sklookup prog to `pod1` netns where the python server is running

```
sudo ./.output/attach-sklookup /sys/fs/bpf/sk_lookup /sys/fs/bpf/sk_lookup_link pod1

```

7. Ensure that the server can be reached from port `8789` (even though it's running 
    in the pod at port `8000`)

```
    curl 192.168.10.2:8789
    <!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd">
    <html>
    ...
    </html>
``` 

See sk_lookup program logs 

```
    sudo cat /sys/kernel/debug/tracing/trace_pipe

    <...>-18104   [007] d.s31 97022.594088: bpf_trace_printk: Saw socket lookup attempt With IP    : 20aa8c0 and port: 2255 
```

8. Cleanup 

 - Detach skops program 

```
    sudo bpftool cgroup detach "/sys/fs/cgroup" sock_ops name load_sock
```
 - Remove pinned files 

```
    sudo rm /sys/fs/bpf/load_sock
    sudo rm /sys/fs/bpf/sk_lookup
    sudo rm /sys/fs/bpf/sk_lookup_link
    sudo rm /sys/fs/bpf/socket_map
```