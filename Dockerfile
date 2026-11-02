FROM rust:{{.Inputs.rust_version.Value}}-bookworm AS build
WORKDIR /src

COPY . .
# The cache mounts keep downloaded crates and compiled dependencies between builds
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release \
    && mkdir /out \
    && cp target/release/{{.Inputs.project_name.Value}} /out/{{.Inputs.project_name.Value}}

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /out/{{.Inputs.project_name.Value}} /{{.Inputs.project_name.Value}}

EXPOSE 8080
ENTRYPOINT ["/{{.Inputs.project_name.Value}}"]
