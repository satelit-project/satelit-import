# Docker

**DEPRECATED**

It's recommended that PostgreSQL is running inside a Docker container. The image can be found [in the repository](https://github.com/satelit-project/satelit-import/packages/29066).

To run integration tests Docker is required.

Everything related to Docker and to running the app in Docker can be found in [docker](./) directory.

## Updating images

Currently, there're 3 images that are used by the app:

1. [`import-db`](https://github.com/satelit-project/satelit-import/packages/29066) – contains PostgreSQL database suitable for development and integration tests. To build and push new image run:

```bash
# build new image
docker build --file docker/tests/postgres.Dockerfile --tag import-db --squash .

# create alias with registry path
docker tag import-db docker.pkg.github.com/satelit-project/satelit-import/import-db

# push to github registry
docker push docker.pkg.github.com/satelit-project/satelit-import/import-db:latest
```

2. [`import-serve`](https://github.com/satelit-project/satelit-import/packages/29042) – contains NGINX which serves files from [static/tests/anidb-index](../static/tests/anidb-index) directory. Those files are AniDB dump samples and are used by integration tests. To build and push new image run:

```bash
# build new image
docker build --file docker/tests/nginx.Dockerfile --tag import-serve --squash .

# create alias with registry path
docker tag import-serve docker.pkg.github.com/satelit-project/satelit-import/import-serve

# push to github registry
docker push docker.pkg.github.com/satelit-project/satelit-import/import-serve:latest
```

3. [`import-tests`](https://github.com/satelit-project/satelit-import/packages/28797) – a container to build and run unit and integration tests. To build and push new image run:

```bash
# build new image
docker build --file docker/tests/app.Dockerfile --tag import-tests --squash .

# create alias with registry path
docker tag import-tests docker.pkg.github.com/satelit-project/satelit-import/import-tests

# push to github registry
docker push docker.pkg.github.com/satelit-project/satelit-import/import-tests:latest
```

Remove `--squash` argument if your Docker daemon does not allow experimental features.

## Running standalone containers

You can run those images as a standalone containers. This may be useful for development. For example, you could run `import-db` container to quickly setup PostgreSQL database which is required to run the app. In addition, you could run `import-serve` container to serve some content from [static/tests](../static/tests) directory which is required by some integration tests.

Following commands assume that previously mentioned images has beed pulled from repo's registry and they have short-named aliases in addition to fully qualified name (registry path + repo path + tag). But you can replace those aliases with any tag you want.

Here's how you could run those containers:

1. `import-db`:

```bash
docker run -p 5432:5432 import-db:latest
```

2. `import-serve`:

```bash
docker run -p 8081:8081 --mount "type=bind,source=$(pwd)/static/tests/anidb-index,destination=/usr/share/nginx/anidb" import-serve:latest
```

3. `import-tests`:

```bash
docker run --mount "type=bind,source=$(pwd),destination=/satelit-import/repo" import-tests:latest
```

You can pass `-it` option to any of the `run` command to run interactive session (with `bash` as entrypoint). Moreover, `-d` option may be passed to run it in detached mode (not recommended for tests).

_TODO:_ use same network for containers so `import-tests` could access DB and NGINX.
