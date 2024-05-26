FROM debian:bullseye-slim

ARG PGDATABASE=gis
ARG PGUSER=osm
ARG PGPASSWORD=osm
ENV PGDATABASE=${PGDATABASE}
ENV PGUSER=${PGUSER}
ENV PGPASSWORD=${PGPASSWORD}

RUN apt-get update && \
    apt-get install -y \
        unattended-upgrades \
        git=1:2.30.2-1+deb11u2 \
        osm2pgsql=1.4.1+ds-2 \
        git \
        wget
# Stay up to date with security updates
RUN unattended-upgrade

# Setup user
RUN adduser --system --group ${PGUSER}
USER ${PGUSER}

# Download style and transform script
RUN git clone https://github.com/gravitystorm/openstreetmap-carto.git /home/${PGUSER}/osm-carto
ENV OSM_CARTO_DIR=/home/${PGUSER}/osm-carto

# Download an OSM data set
RUN mkdir -p /home/${PGUSER}/osm-data
ENV OSM_DATA_DIR=/home/${PGUSER}/osm-data
WORKDIR ${OSM_DATA_DIR}
RUN wget -c https://download.geofabrik.de/australia-oceania/australia-240501.osm.pbf

# Ingest the OSM data into Postgresql
CMD osm2pgsql \
    --slim \
    --host 172.20.0.2 \
    --port 5432 \
    --database "${PGDATABASE}" \
    --username "${PGUSER}" \
    --hstore \
    --multi-geometry \
    --number-processes 4 \
    --tag-transform-script "${OSM_CARTO_DIR}/openstreetmap-carto.lua" \
    --style "${OSM_CARTO_DIR}/openstreetmap-carto.style" \
    --cache 8000 \
    ${OSM_DATA_DIR}/*.osm.pbf
