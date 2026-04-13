FROM alpine:edge

RUN apk add --no-cache \
    alpine-sdk \
    cargo \
    rust \
    cmake \
    git \
    linux-headers \
    mesa-dev \
    pkgconf \
    qt5-qtbase-dev \
    qt5-qtdeclarative-dev

WORKDIR /work/packaging/postmarketos

CMD ["/bin/sh"]
