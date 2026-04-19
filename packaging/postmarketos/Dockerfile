FROM alpine:edge

RUN apk add --no-cache \
    alpine-sdk \
    cargo \
    rust \
    clang-dev \
    clang-libclang \
    cmake \
    git \
    linux-headers \
    mesa-dev \
    pkgconf \
    python3 \
    qt5-qtbase-dev \
    qt5-qtdeclarative-dev \
    samurai

WORKDIR /work/packaging/postmarketos

CMD ["/bin/sh"]
