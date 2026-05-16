#!/bin/sh
set -eu

export BOARD_API_HOST="${BOARD_API_HOST:-board-api:3000}"
export FRONTEND_HOST="${FRONTEND_HOST:-frontend:80}"
export PORT="${PORT:-80}"

envsubst '${BOARD_API_HOST} ${FRONTEND_HOST} ${PORT}' \
  < /etc/nginx/nginx.conf.template \
  > /etc/nginx/conf.d/default.conf

exec "$@"
