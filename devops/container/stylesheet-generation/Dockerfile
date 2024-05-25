FROM httpd:2.4.57-bullseye

ARG PGDATABASE=osm
ARG PGUSER=osm
ARG PGPASSWORD=osm
ENV PGDATABASE=${PGDATABASE}
ENV PGUSER=${PGUSER}
ENV PGPASSWORD=${PGPASSWORD}

# Install the dependencies
RUN apt-get update && \
    apt-get install -y \
        unattended-upgrades \
        curl \
        git \
        unzip \
        npm=7.5.2+ds-2 \
        nodejs=12.22.12~dfsg-1~deb11u4 \
        gdal-bin=3.2.2+dfsg-2+deb11u2 \
        mapnik-utils=3.1.0+ds-1 \
        libmapnik-dev=3.1.0+ds-1 \
        python3=3.9.2-3 \
        python3-psycopg2=2.8.6-2 \
        python3-yaml=5.3.1-5 \
        python3-requests=2.25.1+dfsg-2

# Install the carto and psycopg2 packages
RUN npm install -g carto

# Stay up to date with security updates
RUN unattended-upgrade

# Setup user
RUN adduser --system --group ${PGUSER}
USER ${PGUSER}

# Generate the Mapnik stylesheet
RUN git clone https://github.com/gravitystorm/openstreetmap-carto.git /home/${PGUSER}/osm-carto
ENV OSM_CARTO_DIR=/home/${PGUSER}/osm-carto
WORKDIR ${OSM_CARTO_DIR}
RUN git checkout v5.3.1
RUN cat project.mml | sed "s/dbname: \"gis\"/dbname: \"${PGDATABASE}\"/g" > project.osm.mml

# Generate the stylesheet
CMD python3 scripts/get-external-data.py \
    --host 172.20.0.2 \
    --port 5432 \
    --database "${PGDATABASE}" \
    --username "${PGUSER}" \
    --password "${PGPASSWORD}" \
    --verbose \
    && carto --output mapnik --file style.xml project.osm.mml
