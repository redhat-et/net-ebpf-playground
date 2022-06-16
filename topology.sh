#!/bin/sh
set -xe

up() {
	iface="$1"
	if [ -z "$iface" ]; then
		echo "external interface name required"
		exit 1
	fi
	ip link add podrouter type dummy
	ip link set up podrouter
	ip addr add 192.168.10.1/32 dev podrouter
	firewall-cmd --permanent --new-zone=playground
	firewall-cmd --permanent --zone=playground --set-target ACCEPT
	firewall-cmd --reload
	firewall-cmd --zone=playground --change-interface podrouter
	firewall-cmd --direct --add-rule ipv4 nat POSTROUTING 0 -o "$iface" -j MASQUERADE

	create_netns 1 "$iface" 192.168.10.2
	create_pod 1

	create_netns 2 "$iface" 192.168.10.3
	create_pod 2

	create_netns 3 "$iface" 192.168.10.4
	create_pod 3
}

down() {
	set +e
	runc kill --all pod1 KILL
	runc kill --all pod2 KILL
	runc kill --all pod3 KILL
	runc delete pod1
	runc delete pod2
	runc delete pod3
	rm -rf ./containers
	ip netns del pod1
	ip netns del pod2
	ip netns del pod3
	ip link del podrouter
	firewall-cmd --delete-zone=playground --permanent
	firewall-cmd --reload
}

create_netns() {
	pod="pod$1"
	iface="$2"
	ip="$3"
	ip netns add "$pod"
	ip link add "$pod" type veth peer name "$pod"-int
	ip link set "$pod"-int netns "$pod" name eth0
	ip netns exec "$pod" ip addr add "${ip}"/32 dev eth0
	ip netns exec "$pod" ip link set up eth0
	ip netns exec "$pod" ip route add 192.168.10.1 dev eth0
	ip netns exec "$pod" ip route add default via 192.168.10.1
	ip link set up "$pod"
	ip route add "$ip"/32 dev "$pod"
	firewall-cmd --zone=playground --change-interface="$pod"
	# Being in the same zone should be sufficient...
	# but I don't claim to understand firewalld
	firewall-cmd --direct --add-rule ipv4 filter FORWARD 0 -i "$pod" -j ACCEPT
	firewall-cmd --direct --add-rule ipv4 filter FORWARD 1 -i "$iface" -o "$pod" -m state --state RELATED,ESTABLISHED -j ACCEPT
	firewall-cmd --direct --add-rule ipv4 filter FORWARD 2 -i "$iface" -o "$pod" -j ACCEPT
}

create_pod() {
	pod="pod$1"
	if [ ! -f .output/config.json ]; then
		mkdir -p .output
		skopeo copy docker://alpine:3.16 oci:alpine:3.16
		umoci unpack --image alpine:3.16 ./.output
		rm -rf alpine
	fi
	if [ ! -f .output/resolv.conf ]; then
		echo "nameserver 1.1.1.1" > .output/resolv.conf
	fi
	bundle_dir="./containers/${pod}"
	mkdir -p "${bundle_dir}"
	cp -rf .output/* "${bundle_dir}"
	jq --arg pod "$pod" '.linux.namespaces[1].path = "/var/run/netns/\($pod)" | .process.args[0] = "sleep" | .process.args[1] = "infinity" | .process.terminal = false | .root.readonly = false | .hostname = "\($pod)" | .mounts += [{"destination":"/etc/resolv.conf","type":"bind","source":"resolv.conf","options":["ro","rbind","rprivate","nosuid","noexec","nodev"]}]' .output/config.json > "${bundle_dir}/config.json"
	runc --systemd-cgroup create -b "${bundle_dir}" "${pod}"
	runc --systemd-cgroup start "${pod}"
}

clean() {
	rm -rf .output
}

if test "$(id -u)" -ne "0"; then
	echo "please run as root"
	exit 1
fi

case $1 in
	"up")
		up "$2"
		;;
	"down")
		down
		;;
	"clean")
		clean
		;;
	*)
		echo "please provide a command. 'up' or 'down'"
		exit 1
esac
