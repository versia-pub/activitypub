## A simple rust project for microservices using rust, actix, and postgres

This project contains a simple rust project for microservices using rust, actix, and postgres.

The `Dockerfile` in this project is used to build a docker image for the rust project, the output Docker image is a from scratch image that contains the rust binary.

### Building and running the docker image

To build the docker image, run the following command:

```bash
> docker build -t f:latest .
```

To run the docker image, run the following command:

```bash
docker run -i -e RUST_LOG="debug" -e DATABASE_URL="postgresql://postgres:postgres@host.docker.internal:5432/database" -e LISTEN="0.0.0.0:8080" -p 8080:8080 f:latest
```
