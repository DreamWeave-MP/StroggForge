FROM quay.io/centos/centos:stream9

LABEL org.opencontainers.image.title="StroggForge Portmaster ARM64 EL9 Release Builder" \
      org.opencontainers.image.description="Portmaster ARM64 release builder for StroggForge: clang and lld from CentOS Stream 9 cross-compiling for aarch64-unknown-linux-gnu against a glibc 2.34 EL9 aarch64 sysroot." \
      org.opencontainers.image.source="https://github.com/DreamWeave-MP/StroggForge" \
      org.opencontainers.image.vendor="DreamWeave-MP" \
      org.opencontainers.image.licenses="MIT"

# The same CentOS Stream 9 base and clang/lld as the native Linux builder, so Portmaster builds
# under the LLVM toolchain policy too. clang cross-compiles; the sysroot supplies the aarch64
# glibc 2.34, libgcc, and libstdc++ to link against. Package scripts are skipped because they
# are aarch64 binaries this x86_64 host cannot run; EL9's merged /usr layout needs no fix-ups.
# gcc and gcc-c++ are there for their target files (crtbegin.o, libgcc.a, libgcc_s.so, the
# libstdc++ headers), which clang finds under usr/lib/gcc/aarch64-redhat-linux.
RUN dnf --installroot=/usr/aarch64-linux-gnu/sys-root --releasever=9 --forcearch=aarch64 \
        --setopt=tsflags=noscripts --setopt=install_weak_deps=False install -y \
        glibc-devel libgcc libstdc++-devel gcc gcc-c++ openssl-devel \
    && dnf --installroot=/usr/aarch64-linux-gnu/sys-root clean all \
    && test -f /usr/aarch64-linux-gnu/sys-root/usr/lib64/crt1.o \
    && test -f /usr/aarch64-linux-gnu/sys-root/usr/lib64/libgcc_s.so.1 \
    && test -d /usr/aarch64-linux-gnu/sys-root/usr/lib/gcc/aarch64-redhat-linux \
    && test -f /usr/aarch64-linux-gnu/sys-root/usr/include/openssl/ssl.h \
    && test -f /usr/aarch64-linux-gnu/sys-root/usr/lib64/pkgconfig/openssl.pc \
    && dnf install -y epel-release \
    && dnf install -y \
        git \
        python3 \
        make \
        gettext \
        pkgconf-pkg-config \
        p7zip \
        p7zip-plugins \
        clang \
        lld \
        llvm \
    && mkdir -p /home/runner/work \
    && { [ -e /home/runner/work/_actions ] || ln -s /__w/_actions /home/runner/work/_actions; } \
    && dnf clean all \
    && rm -rf /var/cache/dnf
