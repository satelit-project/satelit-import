# App

Docker image to run `satelit-import` service in production.

## Building

To build new version of the image run the following command:

``` sh
VERSION="<version>"
docker build -t satelit/satelit-import:"$VERSION" -f docker/app/Dockerfile .
docker push satelit/satelit-import:"$VERSION"
```
