## Lysand ActivityPub Layer

A simple activitypub compatibility layer ("bridge"), to make Lysand compatible with the ActivityPub and ActivityStreams protocol.
The layer is realised in a microservice format.

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

### Building and running the docker image

To build the docker image, run the following command:

```bash
> docker build -t f:latest .
```

To run the docker image, run the following command:

```bash
docker run -i -e RUST_LOG="debug" -e DATABASE_URL="postgresql://postgres:postgres@host.docker.internal:5432/database" -e LISTEN="0.0.0.0:8080" -p 8080:8080 f:latest
```
