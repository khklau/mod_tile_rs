FROM postgis/postgis:13-3.4

ARG POSTGRES_DB=gis
ARG POSTGRES_USER=osm
ARG POSTGRES_PASSWORD=osm
ARG POSTGRES_RUN_DIR=/run/postgresql
ENV POSTGRES_DB=${POSTGRES_DB}
ENV POSTGRES_USER=${POSTGRES_USER}
ENV POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
ENV POSTGRES_RUN_DIR=${POSTGRES_RUN_DIR}

RUN apt-get update && \
    apt-get install -y unattended-upgrades
# Stay up to date with security updates
RUN unattended-upgrade

RUN adduser --system --group ${POSTGRES_USER}

COPY createdb_osm.sh /docker-entrypoint-initdb.d/20_createdb_osm.sh
COPY initdb_osm.sql /docker-entrypoint-initdb.d/30_initdb_osm.sql
COPY postgresql.conf /etc/postgresql/postgresql.conf

RUN rm -rf ${POSTGRES_RUN_DIR}
VOLUME ${POSTGRES_RUN_DIR}
