# syntax=docker/dockerfile:1

# ---------- frontend ----------
FROM node:26-alpine AS frontend
ARG HTTP_PROXY
ARG HTTPS_PROXY
ARG http_proxy
ARG https_proxy
ARG ALL_PROXY
ARG all_proxy
ARG NO_PROXY
ARG no_proxy
WORKDIR /web
COPY frontend/h5/package.json frontend/h5/yarn.lock ./
RUN yarn --version && yarn install --frozen-lockfile
COPY frontend/h5/ ./
RUN yarn build

# ---------- backend ----------
FROM rust:1.95-alpine AS backend
ARG HTTP_PROXY
ARG HTTPS_PROXY
ARG http_proxy
ARG https_proxy
ARG ALL_PROXY
ARG all_proxy
ARG NO_PROXY
ARG no_proxy
RUN apk add --no-cache build-base perl
WORKDIR /build
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release --locked

# ---------- runtime ----------
FROM nginx:1.31-alpine
RUN rm -f /etc/nginx/conf.d/default.conf
WORKDIR /app
COPY nginx/nginx.conf /etc/nginx/conf.d/default.conf
COPY backend_startup.sh /backend_startup.sh
COPY --from=frontend /web/dist/ /usr/share/nginx/html/
COPY --from=backend /build/target/release/dir-vigil /usr/local/bin/dir-vigil

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
  CMD wget -qO- http://127.0.0.1/api/status >/dev/null || exit 1

CMD sh -c "nginx -g 'daemon off;' & /backend_startup.sh"
