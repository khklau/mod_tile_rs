#!/bin/sh

mkdir -p /shared/run/renderd
mkdir -p /shared/cache/renderd/tiles
chown -R ${PGUSER}:${RENDER_GROUP_NAME} /shared/run/renderd
chown -R ${PGUSER}:${RENDER_GROUP_NAME} /shared/cache/renderd/tiles

/usr/bin/renderd --foreground --config /etc/renderd.conf
