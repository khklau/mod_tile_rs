#!/bin/sh -x

if [ -n "$(docker container ps --all | awk '{print $2}' | grep '^aus-osm-postgis:13-3.4')" ]; then
	docker container stop aus-osm-postgis
	docker container rm aus-osm-postgis
fi

docker container run --name aus-osm-postgis --network=osm-local --detach -p 5432:5432 aus-osm-postgis:13-3.4

