# syntax=docker/dockerfile:1
# Edge reverse proxy (контекст ./proxy)
FROM nginx:1.27-alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
