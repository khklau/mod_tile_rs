#!/bin/sh -x

docker network create osm-local --subnet=172.20.0.0/16
docker run --name osm-postgis --network=osm-local --detach -p 5432:5432 osm-postgis:13-3.4
sleep 10
docker run --name osm-data-ingestion --network=osm-local osm-data-ingestion:1.4.1
#docker network rm osm-local

