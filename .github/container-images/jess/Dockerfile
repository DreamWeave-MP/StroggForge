# The jess image: jess for Linux x86_64 and every platform's jess as a runtime to pack with, on
# nothing but the libraries the Linux jess links against. Built by publishJessImage.yml from the
# release archives the calling jess release built in the same run, so what it holds is what that
# release ships. The build context holds:
#
#   runtimes/jess-<System>-<Arch>[.exe]   each release archive's binary, named for its archive
#   licenses/                             the release's LICENSE-* and README
#
# Pipelines run it as a step (`uses: docker://ghcr.io/dreamweave-mp/jess@sha256:...`): checkout and
# signing stay on the runner, so it needs no shell, no git, and no package manager.

FROM quay.io/centos/centos:stream9 AS libraries

COPY runtimes/ /opt/jess/runtimes/

# The release's Linux jess is built against this stream's glibc 2.34, so its libraries come from
# here, copied with ldd's own list, link targets resolved. The CA bundle is for TLS: the platform
# verifier reads the system store.
RUN dnf install -y libstdc++ ca-certificates \
    && test -x /opt/jess/runtimes/jess-Linux-X64 \
    && mkdir -p /rootfs/tmp /rootfs/etc/ssl/certs /rootfs/usr/local/bin \
    && chmod 1777 /rootfs/tmp \
    && ldd /opt/jess/runtimes/jess-Linux-X64 \
        | awk '$2 == "=>" && $3 ~ /^\// { print $3 } $1 ~ /^\// { print $1 }' \
        | while read -r library; do \
            mkdir -p "/rootfs$(dirname "$library")" && cp -L "$library" "/rootfs$library"; \
        done \
    && test -f /rootfs/lib64/ld-linux-x86-64.so.2 \
    && cp -L /etc/pki/tls/certs/ca-bundle.crt /rootfs/etc/ssl/certs/ca-certificates.crt \
    && ln -s /opt/jess/runtimes/jess-Linux-X64 /rootfs/usr/local/bin/jess

FROM scratch

LABEL org.opencontainers.image.title="jess" \
      org.opencontainers.image.description="jess, the native application kernel for Luau applications, with every platform's jess as a runtime to pack standalone executables with, and nothing else." \
      org.opencontainers.image.source="https://github.com/DreamWeave-MP/jess" \
      org.opencontainers.image.vendor="DreamWeave-MP" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"

COPY --from=libraries /rootfs/ /
COPY runtimes/ /opt/jess/runtimes/
COPY licenses/ /usr/share/licenses/jess/

ENV PATH=/usr/local/bin \
    HOME=/tmp \
    SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
    JESS_RUNTIMES=/opt/jess/runtimes

# The exec form needs no shell: this fails the build when a library is missing.
RUN ["/usr/local/bin/jess", "--version"]

ENTRYPOINT ["/usr/local/bin/jess"]
