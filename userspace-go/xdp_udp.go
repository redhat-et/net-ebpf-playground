package main

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"regexp"
	"strings"

	"github.com/cilium/ebpf"
	"github.com/cilium/ebpf/link"
)

//go:generate go run github.com/cilium/ebpf/cmd/bpf2go -cc $BPF_CLANG -cflags $BPF_CFLAGS bpf ../bpf/xdp_udp.bpf.c -- -I../libbpf/src
func main() {
	if len(os.Args) < 3 {
		log.Fatalf("Please specify a main and destination network interface")
	}

	ifaceName := os.Args[1]
	iface, err := net.InterfaceByName(ifaceName)
	if err != nil {
		log.Fatalf("lookup network iface %q: %s", ifaceName, err)
	}
	ifaceName2 := os.Args[2]
	iface2, err := net.InterfaceByName(ifaceName2)
	if err != nil {
		log.Fatalf("lookup network iface %s: %s", ifaceName, err)
	}

	objs := bpfObjects{}
	if err := loadBpfObjects(&objs, nil); err != nil {
		log.Fatalf("loading objects: %s", err)
	}
	defer objs.Close()

	l, err := link.AttachXDP(link.XDPOptions{
		Program:   objs.XdpProgFunc,
		Interface: iface.Index,
	})
	if err != nil {
		log.Fatalf("could not attach XDP program: %s", err)
	}
	defer l.Close()

	l2, err := link.AttachXDP(link.XDPOptions{
		Program:   objs.BpfRedirectPlaceholder,
		Interface: iface2.Index,
	})
	if err != nil {
		log.Fatalf("could not attach XDP program: %s", err)
	}
	defer l2.Close()

	log.Printf("Attached XDP program to iface %q (index %d)", iface.Name, iface.Index)
	log.Printf("Press Ctrl-C to exit and remove the program")

	b := bpfBackend{
		Saddr: ip2int("10.8.125.12"),
		Daddr: ip2int("192.168.10.2"),
		Dport: 9875,
		// Host-Side Veth Mac
		Shwaddr: hwaddr2bytes("06:56:87:ec:fd:1f"),
		// Container-Side Veth Mac
		Dhwaddr: hwaddr2bytes("86:ad:33:29:ff:5e"),
		Nocksum: 0,
		Ifindex: 8,
	}

	key := bpfVipKey{
		Vip: ip2int("10.8.125.12"),
		//Vip:  ip2int("192.168.10.1"),
		Port: 8888,
	}

	if err := objs.Backends.Update(key, b, ebpf.UpdateAny); err != nil {
		fmt.Println(err.Error())
		os.Exit(1)
	}

	for {
	}
}

func ip2int(ip string) uint32 {
	ipaddr := net.ParseIP(ip)
	return binary.LittleEndian.Uint32(ipaddr.To4())
}

// feed from interfaces2hwaddr
func hwaddr2bytes(hwaddr string) [6]byte {
	parts := strings.Split(hwaddr, ":")
	if len(parts) != 6 {
		panic("invalid hwaddr")
	}

	var hwaddrB [6]byte
	for i, hexPart := range parts {
		bs, err := hex.DecodeString(hexPart)
		if err != nil {
			panic(err)
		}
		if len(bs) != 1 {
			panic("invalid hwaddr part")
		}
		hwaddrB[i] = bs[0]
	}

	return hwaddrB
}

type networkInterface struct {
	name    string
	hwaddr  [6]uint8
	ifindex uint16
}

// interface to hwaddr in hex
func interfaces2hwaddr() (interfaces map[string]networkInterface) {
	ints, err := net.Interfaces()
	if err != nil {
		panic(err)
	}

	for _, in := range ints {
		interfaces[in.Name] = networkInterface{
			name:    in.Name,
			hwaddr:  hwaddr2bytes(in.HardwareAddr.String()),
			ifindex: uint16(in.Index),
		}
	}

	return
}

var routeRE = regexp.MustCompile(`^([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+)(/[0-9]+)? dev (\S+)`)

// ip to interface name
func routes() (routes map[string]string) {
	stdout, stderr := new(bytes.Buffer), new(bytes.Buffer)
	cmd := exec.Command("ip", "route")
	cmd.Stdout = stdout
	cmd.Stderr = stderr
	if err := cmd.Run(); err != nil {
		panic(err.Error() + stderr.String())
	}

	for _, line := range strings.Split(stdout.String(), "\n") {
		matches := routeRE.FindAllStringSubmatch(line, -1)
		if len(matches) == 1 {
			submatches := matches[0]
			routes[submatches[1]] = submatches[3]
		}
	}

	return
}
