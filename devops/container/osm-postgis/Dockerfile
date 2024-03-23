FROM postgis/postgis:13-3.4

ENV POSTGRES_DB osm
ENV POSTGRES_USER osm
ENV POSTGRES_PASSWORD osm

RUN apt-get update && \
    apt-get install -y unattended-upgrades
# Stay up to date with security updates
RUN unattended-upgrade

COPY createdb_osm.sh /docker-entrypoint-createdb.d/20_createdb_osm.sh
COPY initdb_osm.sql /docker-entrypoint-initdb.d/30_initdb_osm.sql
