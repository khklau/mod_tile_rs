#!/bin/sh -x

if [ -z "$(docker network ls | grep osm-local)" ]; then
	docker network create osm-local --subnet=172.20.0.0/16
fi

if [ -n "$(docker container ps --all | awk '{print $2}' | grep '^osm-postgis:13-3.4')" ]; then
	docker container stop osm-postgis
	docker container rm osm-postgis
fi

if [ -n "$(docker container ps --all | awk '{print $2}' | grep '^osm-data-ingestion:1.4.1')" ]; then
	docker container stop osm-data-ingestion
	docker container rm osm-data-ingestion
fi

# Create a PostGIS container loaded with OSM data
if [ -z "$(docker image ls | grep aus-osm-postgis)" ]; then
	docker container stop aus-osm-postgis
	docker container rm aus-osm-postgis
	docker container run --name osm-postgis --network=osm-local --detach -p 5432:5432 osm-postgis:13-3.4 \
		&& sleep 15 \
		&& docker container run --name osm-data-ingestion --network=osm-local osm-data-ingestion:1.4.1 \
		&& docker container commit $(docker container ps | grep osm-postgis:13-3.4 | awk '{print $1}') aus-osm-postgis:13-3.4 \
		&& docker container stop osm-postgis \
		&& docker container rm osm-postgis
fi

docker container run --name aus-osm-postgis --network=osm-local --detach -p 5432:5432 aus-osm-postgis:13-3.4

