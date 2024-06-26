#!/bin/sh -x

CONTAINER_DIR=$(dirname $0 | xargs readlink -f)
SHARED_VOL_DIR=${HOME}/shared

if [ -z "$(docker network ls | grep osm-local)" ]; then
    docker network create osm-local --subnet=172.20.0.0/16
fi

if [ -z "$(docker volume ls | grep shared-legacy-renderer)" ]; then
    docker volume create --name shared-legacy-renderer
fi

if [ -z "$(docker volume ls | grep postgis-run)" ]; then
    docker volume create --name postgis-run
fi

if [ -z "$(docker image ls | grep '^osm-postgis' | grep '13-3.4')" ]; then
    docker build --tag osm-postgis:13-3.4 ${CONTAINER_DIR}/osm-postgis
fi

if [ -z "$(docker image ls | grep '^osm-data-ingestion' | grep '1.4.1')" ]; then
    docker build --tag osm-data-ingestion:1.4.1 devops/container/data-ingestion
fi

if [ -z "$(docker image ls | grep '^stylesheet-generation' | grep '5.3.1')" ]; then
    docker build --tag stylesheet-generation:5.3.1 devops/container/stylesheet-generation
fi

# Create a PostGIS container loaded with OSM data
if [ -z "$(docker image ls | grep '^aus-osm-postgis' | grep '13-3.4')" ]; then
    docker container stop osm-data-ingestion
    docker container rm osm-data-ingestion
    docker container stop osm-postgis
    docker container rm osm-postgis
    docker container run --name osm-postgis --rm --volume postgis-run:/run/postgresql --network=osm-local --detach -p 5432:5432 osm-postgis:13-3.4 \
        && sleep 10 \
        && docker container run --name osm-data-ingestion --rm --network=osm-local osm-data-ingestion:1.4.1 \
        && docker container commit $(docker container ps --all | grep osm-postgis:13-3.4 | awk '{print $1}') aus-osm-postgis:13-3.4
    docker container stop osm-postgis
fi

if [ -n "$(docker container ps --all | awk '{print $2}' | grep '^aus-osm-postgis:13-3.4')" ]; then
    docker container stop aus-osm-postgis
    docker container rm aus-osm-postgis
fi
if [ -n "$(docker container ps --all | awk '{print $2}' | grep '^stylesheet-generation:5.3.1')" ]; then
    docker container stop stylesheet-generation
    docker container rm stylesheet-generation
fi
if [ ! -e ${CONTAINER_DIR}/legacy-renderer/style.xml ]; then
    docker container run --name aus-osm-postgis --rm --volume postgis-run:/run/postgresql --network=osm-local --detach -p 5432:5432 aus-osm-postgis:13-3.4
    sleep 10
    docker container run --name stylesheet-generation --network=osm-local stylesheet-generation:5.3.1 \
        && docker cp stylesheet-generation:/home/osm/osm-carto/style.xml ${CONTAINER_DIR}/legacy-renderer/style.xml \
        && docker cp stylesheet-generation:/home/osm/osm-carto/style.xml ${CONTAINER_DIR}/next-renderer/style.xml
    docker container rm stylesheet-generation
fi

# Since legacy-tileserver needs files from multiple directories the build context directory needs to be ${CONTAINER_DIR}
if [ -z "$(docker image ls | grep '^legacy-tileserver' | grep '2.4.57-0.5-2')" ]; then
    HERE=$(pwd)
    cd ${CONTAINER_DIR}
    docker build --tag legacy-tileserver:2.4.57-0.5-2 --file legacy-tileserver/Dockerfile .
    cd ${HERE}
fi

if [ -z "$(docker image ls | grep '^legacy-renderer' | grep '0.5-2')" ]; then
    docker build --tag legacy-renderer:0.5-2 ${CONTAINER_DIR}/legacy-renderer
fi

