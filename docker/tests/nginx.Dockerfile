FROM nginx:alpine

VOLUME ["/usr/share/nginx/anidb"]

COPY docker/tests/nginx/nginx.conf /etc/nginx/nginx.conf
