FROM postgis/postgis:13-3.4

ENV POSTGRES_DB osm
ENV POSTGRES_USER osm
ENV POSTGRES_PASSWORD osm

RUN apt-get update && \
    apt-get upgrade && \
    apt-get install -y osm2pgsql

COPY init_osm.sql /docker-entrypoint-initdb.d/
