#!/bin/sh -x

if [ -n "$(docker network ls | grep osm-local)" ]; then
	docker network rm osm-local
fi
docker network create osm-local --subnet=172.20.0.0/16

# Create a PostGIS container loaded with OSM data
if [ -n "$(docker image ls | grep aus-osm-postgis | grep 13-3.4)" ]; then
	docker run --name osm-postgis --network=osm-local --detach -p 5432:5432 osm-postgis:13-3.4 \
		&& sleep 15 \
		&& docker run --name osm-data-ingestion --network=osm-local osm-data-ingestion:1.4.1 \
		&& docker commit $(docker container ps | grep osm-postgis:13-3.4 | awk '{print $1}') aus-osm-postgis:13-3.4 \
		&& docker container stop osm-postgis
fi

docker run --name aus-osm-postgis --network=osm-local --detach -p 5432:5432 aus-osm-postgis:13-3.4

