
<p align="center">
  <a href="https://lysand.org"><img src="https://cdn.lysand.org/logo-long-dark.webp" alt="Lysand Logo" height="110"></a>
</p>

## Lysand ActivityPub Bridge

[![Nix Flake actions](https://github.com/lysand-org/activitypub/actions/workflows/nix-flake.yml/badge.svg)](https://github.com/lysand-org/activitypub/actions/workflows/nix-flake.yml)

**ActivityPub/ActivityStreams** compatibility layer for [**Lysand Server**](https://github.com/lysand-org/lysand).

Designed as a microservice, runs as its own process and communicates with the main server via HTTP.

## Development (Flakes)

This repo uses [Flakes](https://nixos.wiki/wiki/Flakes) from the get-go.

```bash
# Dev shell
nix develop

# or run via cargo
nix develop -c cargo run

# build
nix build
```

We also provide a [`justfile`](https://just.systems/) for Makefile'esque commands.

## Building

### Docker/Podman

To build the Docker image, run the following command:

```bash
docker build -t activitypub:latest .
```

To run the docker image, use the [`docker-compose.yml`](./docker-compose.yml) file:

```bash
wget https://raw.githubusercontent.com/lysand-org/activitypub/main/docker-compose.yml

docker-compose up -d
```

If you are building from source, make sure to replace the image name in the `docker-compose.yml` file.