FROM clux/muslrust:latest AS builder
WORKDIR /satelit-import/
COPY . .
RUN docker/app/scripts/build-project.sh

FROM alpine:latest
WORKDIR /satelit-import/
EXPOSE 9060
COPY --from=builder /satelit-import/satelit-import.tar.gz /satelit-import/
RUN ["tar", "-xvf", "satelit-import.tar.gz"]
ENTRYPOINT ["/satelit-import/entry.sh"]
HEALTHCHECK --interval=1m --start-period=10s CMD ps cax | grep satelit-import >/dev/null || exit 1
