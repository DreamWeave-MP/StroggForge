FROM almalinux:8

LABEL org.opencontainers.image.title="StroggForge Portmaster ARM64 AlmaLinux 8 Release Builder" \
      org.opencontainers.image.description="Portmaster ARM64 release builder for StroggForge with a glibc 2.28 aarch64-unknown-linux-gnu sysroot on AlmaLinux 8." \
      org.opencontainers.image.source="https://github.com/DreamWeave-MP/StroggForge" \
      org.opencontainers.image.vendor="DreamWeave-MP" \
      org.opencontainers.image.licenses="MIT"

RUN dnf install -y epel-release \
    && { dnf --installroot=/usr/aarch64-linux-gnu/sys-root --releasever=8 --forcearch=aarch64 --setopt=tsflags=noscripts --setopt=install_weak_deps=False install -y glibc-devel libstdc++-devel libgcc openssl-devel \
    || { test -f /usr/aarch64-linux-gnu/sys-root/usr/lib64/crt1.o && test -f /usr/aarch64-linux-gnu/sys-root/lib64/libgcc_s.so.1 && test -f /usr/aarch64-linux-gnu/sys-root/usr/include/openssl/ssl.h && test -f /usr/aarch64-linux-gnu/sys-root/usr/lib64/pkgconfig/openssl.pc; }; } \
    && ln -sf ../usr/lib64/libc.so.6 /usr/aarch64-linux-gnu/sys-root/lib64/libc.so.6 \
    && ln -sf ../usr/lib64/ld-2.28.so /usr/aarch64-linux-gnu/sys-root/lib64/ld-2.28.so \
    && ln -sf libgcc_s.so.1 /usr/aarch64-linux-gnu/sys-root/lib64/libgcc_s.so \
    && dnf install -y \
        git \
        python3 \
        make \
        gettext \
        pkgconf-pkg-config \
        p7zip \
        p7zip-plugins \
        clang \
        clang-devel \
        gcc-aarch64-linux-gnu \
        gcc-c++-aarch64-linux-gnu \
        binutils-aarch64-linux-gnu \
    && mkdir -p /home/runner/work \
    && { [ -e /home/runner/work/_actions ] || ln -s /__w/_actions /home/runner/work/_actions; } \
    && dnf clean all \
    && rm -rf /var/cache/dnf
