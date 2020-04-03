FROM rust:1.43 as build
ARG BUILD_TARGET="--release"

# check base image dependencies
RUN command -v curl 2>&1 >/dev/null \
    || { echo "error: curl is required for the Docker healthcheck"; false; } \
 && true
  # ^ append more checks here

COPY extra/docker/entrypoint.sh /opt
RUN chmod a+x /opt/entrypoint.sh

# pre-cache dependencies in a separate layer,
# see: https://github.com/rust-lang/cargo/issues/2644

WORKDIR /usr/src/octoplex

# Create blank project
RUN USER=root cargo init ./

# We want dependencies cached, so copy those first
COPY ./Cargo.toml ./Cargo.lock ./

# This is a dummy build to get the dependencies cached
RUN cargo fetch --locked \
 && cargo build --frozen ${BUILD_TARGET}

# Now copy in the rest of the sources
COPY ./src ./src
COPY ./tests ./tests

# This is the actual build
RUN touch ./src/main.rs \
 && cargo build --frozen ${BUILD_TARGET} \
 && mkdir -p /opt/octoplex/bin \
 && find ./target -maxdepth 2 -name 'octoplex' -type f -exec cp {} /opt/octoplex/bin \;

EXPOSE 8080

ENTRYPOINT ["/opt/entrypoint.sh"]
CMD ["octoplex-dev"]

################################################################################
FROM rust:1.43 as binary

COPY --from=build /opt /opt

WORKDIR /opt/octoplex

EXPOSE 8080

ENTRYPOINT ["/opt/entrypoint.sh"]
CMD ["octoplex"]
