FROM almalinux:8

LABEL org.opencontainers.image.title="StroggForge Native Linux AlmaLinux 8 Release Builder" \
      org.opencontainers.image.description="Native Linux x86_64 release builder for StroggForge targeting glibc 2.28 compatibility on AlmaLinux 8." \
      org.opencontainers.image.source="https://github.com/DreamWeave-MP/StroggForge" \
      org.opencontainers.image.vendor="DreamWeave-MP" \
      org.opencontainers.image.licenses="MIT"

RUN dnf install -y epel-release \
    && dnf install -y \
        git \
        python3 \
        gcc \
        gcc-c++ \
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
