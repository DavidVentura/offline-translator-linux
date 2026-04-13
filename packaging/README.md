# Packaging

## Ubuntu Touch

The repo contains a `clickable/` project for building a `.click` package.

Build the local click artifact with:

```bash
cd clickable
./package-click.sh
```

This builds the Rust binary in release mode, copies it into the Clickable project as
`translator`, and runs `clickable build`.

## postmarketOS

The repo contains a starter `APKBUILD` in `packaging/postmarketos/`.

This is meant as a local recipe scaffold or as a starting point for a `pmaports` merge request.
Before upstreaming it, you should:

1. Set the final `license`.
2. Make sure `url` and `source` point at your released source tarball.
3. Vendor the Cargo git dependencies (`bergamot-sys` and `rust-cld2`) into that tarball or package them separately.

The native distribution artifact on postmarketOS is an Alpine `.apk`, built from this recipe.

For local Docker-based builds, use:

```bash
./packaging/postmarketos/build-apk-docker.sh
```

The container definition lives in `packaging/postmarketos/Dockerfile`.
