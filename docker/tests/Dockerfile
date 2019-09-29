FROM ubuntu:rolling

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH 

WORKDIR /satelit-import

VOLUME ["/satelit-import/repo"]

SHELL ["/bin/bash", "-c"]

COPY docker/tests/support/* .

RUN ./provision.sh

CMD ./run-tests.sh
