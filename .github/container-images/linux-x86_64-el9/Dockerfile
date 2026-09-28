FROM quay.io/centos/centos:stream9

LABEL org.opencontainers.image.title="StroggForge Native Linux EL9 Release Builder" \
      org.opencontainers.image.description="Native Linux x86_64 release builder for StroggForge targeting glibc 2.34 (RHEL/AlmaLinux 9) compatibility, with the clang and lld the EL9 stream ships for the LLVM toolchain policy." \
      org.opencontainers.image.source="https://github.com/DreamWeave-MP/StroggForge" \
      org.opencontainers.image.vendor="DreamWeave-MP" \
      org.opencontainers.image.licenses="MIT"

# CentOS Stream 9 rather than AlmaLinux 9: it is the same glibc 2.34 / libstdc++ 11 ABI, and it
# carries the current clang and lld release (22.x at the time of writing) that l3i and every crate
# depending on it require, one minor release before AlmaLinux does. The official LLVM tarballs
# need GCC 12's libstdc++ and cannot run on any EL9, and AlmaLinux 8 cannot run LLVM 22 at all,
# which is why the glibc 2.28 target was given up.
RUN dnf install -y epel-release \
    && dnf install -y \
        git \
        python3 \
        gcc \
        gcc-c++ \
        clang \
        lld \
        make \
        gettext \
        glibc-devel \
        pkgconf-pkg-config \
        openssl-devel \
        p7zip \
        p7zip-plugins \
    && mkdir -p /home/runner/work \
    && { [ -e /home/runner/work/_actions ] || ln -s /__w/_actions /home/runner/work/_actions; } \
    && dnf clean all \
    && rm -rf /var/cache/dnf
