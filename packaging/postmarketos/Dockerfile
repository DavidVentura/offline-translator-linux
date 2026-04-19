FROM alpine:edge

RUN apk add --no-cache \
    alpine-sdk \
    bash \
    cargo \
    rust \
    clang-dev \
    clang-libclang \
    cmake \
    git \
    linux-headers \
    mesa-dev \
    pkgconf \
    pulseaudio-dev \
    python3 \
    qt5-qtbase-dev \
    qt5-qtdeclarative-dev \
    samurai

WORKDIR /work/packaging/postmarketos

CMD ["/bin/sh"]
