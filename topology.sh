#!/bin/sh
set -e

up() {
	ip link add podrouter type dummy
	ip link set up podrouter
	ip addr add 192.168.10.1/32 dev podrouter

	create_netns 1 192.168.10.2
	create_pod 1

	create_netns 2 192.168.10.3
	create_pod 2

	create_netns 3 192.168.10.4
	create_pod 3
}

down() {
	set +e
	sudo runc kill --all pod1 KILL
	sudo runc kill --all pod2 KILL
	sudo runc kill --all pod3 KILL
	sudo runc delete pod1
	sudo runc delete pod2
	sudo runc delete pod3
	sudo rm -rf ./containers
	ip netns del pod1
	ip netns del pod2
	ip netns del pod3
	ip link del podrouter
}

create_netns() {
	pod="pod$1"
	ip="$2"
	ip netns add "$pod"
	ip link add "$pod" type veth peer name "$pod"-int
	ip link set "$pod"-int netns "$pod" name eth0
	ip netns exec "$pod" ip addr add "${ip}"/32 dev eth0
	ip netns exec "$pod" ip link set up eth0
	ip netns exec "$pod" ip route add 192.168.10.1 dev eth0
	ip netns exec "$pod" ip route add default via 192.168.10.1
	ip link set up "$pod"
	ip route add "$ip"/32 dev "$pod"
}

create_pod() {
	pod="pod$1"
	if [ ! -f .output/config.json ]; then
		mkdir -p .output
		pushd output
		skopeo copy docker://fedora:36 oci:fedora:36
		umoci unpack --image fedora:36 .output
		popd
	fi
	bundle_dir="./containers/${pod}"
	mkdir -p "${bundle_dir}"
	cp -rf .output/* "${bundle_dir}"
	jq --arg pod "$pod" '.linux.namespaces[1].path = "/var/run/netns/\($pod)" | .process.args[0] = "sleep" | .process.args[1] = "infinity" | .process.terminal = false | .root.readonly = false' .output/config.json > "${bundle_dir}/config.json"
	sudo runc --systemd-cgroup create -b "${bundle_dir}" "${pod}"
	sudo runc --systemd-cgroup start "${pod}"
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
		up
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
