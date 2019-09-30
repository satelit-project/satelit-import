FROM postgres:alpine

ENV POSTGRES_USER=satelit \
    POSTGRES_DB=satelit

COPY docker/tests/postgres/extensions.sh /docker-entrypoint-initdb.d/
