#!/bin/sh -x

if [ -n "$(docker container ps --all | awk '{print $2}' | grep '^aus-osm-postgis:13-3.4')" ]; then
    docker container stop aus-osm-postgis
fi
docker container run \
    --name aus-osm-postgis \
    --detach \
    --rm \
    --volume postgis-run:/run/postgresql \
    --network=osm-local \
    --publish 5432:5432 \
    aus-osm-postgis:13-3.4

if [ -n "$(docker container ps --all | awk '{print $2}' | grep '^legacy-renderer:0.5-2')" ]; then
    docker container stop legacy-renderer
fi
docker container run \
    --name legacy-renderer \
    --detach \
    --rm \
    --volume postgis-run:/run/postgresql \
    --volume shared-legacy-renderer:/shared \
    --network=osm-local \
    legacy-renderer:0.5-2

if [ -n "$(docker container ps --all | awk '{print $2}' | grep '^legacy-tileserver:2.4.57-0.5-2')" ]; then
    docker container stop legacy-tileserver
fi
docker container run \
    --name legacy-tileserver \
    --rm \
    --volume shared-legacy-renderer:/shared \
    --network=osm-local \
    --publish 8080:8080 \
    legacy-tileserver:2.4.57-0.5-2

