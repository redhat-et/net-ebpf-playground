FROM fedora:36

RUN dnf update -y && dnf install -y \
    lksctp-tools \
    curl \
    iperf
