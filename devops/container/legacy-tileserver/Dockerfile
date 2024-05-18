FROM httpd:2.4.57-bullseye

ARG HTTPD_USER=www-data
ARG HTTPD_DIR=/usr/local/apache2/htdocs
ARG PGDATABASE=osm
ARG PGUSER=osm
ARG PGPASSWORD=osm
ENV HTTPD_USER=${HTTPD_USER}
ENV PGDATABASE=${PGDATABASE}
ENV PGUSER=${PGUSER}
ENV PGPASSWORD=${PGPASSWORD}
# Install the dependencies
RUN apt-get update && \
    apt-get install -y \
        unattended-upgrades \
        wget \
        unzip \
        libapache2-mod-tile=0.5-2 \
        renderd=0.5-2 \
        fonts-dejavu \
        fonts-unifont \
        fonts-noto-cjk \
        fonts-noto-cjk-extra \
        fonts-noto-hinted \
        fonts-noto-unhinted

# Stay up to date with security updates
RUN unattended-upgrade

# Setup renderd user
RUN adduser --system --group ${PGUSER}

# Add the configuration files
RUN mv /etc/apache2/ports.conf /etc/apache2/ports.orig.conf
COPY ports.conf /etc/apache2/ports.conf
RUN mv /usr/local/apache2/conf/httpd.conf /usr/local/apache2/conf/httpd.orig.conf
COPY httpd.conf /usr/local/apache2/conf/httpd.conf
RUN mv /etc/apache2/sites-available/000-default.conf /etc/apache2/sites-available/000-default.orig.conf
COPY 000-default.conf /etc/apache2/sites-available/000-default.conf
COPY renderd.conf /etc/renderd.conf
COPY style.xml /etc/renderd-style.xml

# Add the static web resources
COPY index.html ${HTTPD_DIR}/index.html

# Setup directories
WORKDIR ${HTTPD_DIR}
RUN wget http://cdn.leafletjs.com/leaflet/v1.7.1/leaflet.zip
RUN unzip leaflet.zip
RUN rm leaflet.zip
RUN chown -R ${HTTPD_USER} ${HTTPD_DIR}
RUN chown -R ${HTTPD_USER} /usr/local/apache2

RUN mkdir -p /run/renderd
RUN mkdir -p /var/cache/renderd/tiles
RUN chown -R ${PGUSER} /run/renderd
RUN chown -R ${PGUSER} /var/cache/renderd/tiles

# Enable the tileserver site and mod_tile
#RUN a2ensite tileserver_site.conf
RUN a2ensite 000-default.conf
RUN a2enmod tile

# This is actually the tile.conf file, symlinking to avoid confusing
# it with /etc/renderd.conf which is a different configuration file
RUN ln -s /etc/apache2/conf-available/renderd.conf /etc/apache2/conf-available/tile.conf
RUN ln -s /etc/apache2/mods-available/renderd.conf /etc/apache2/mods-available/tile.conf

# Execute https as non-root user
USER ${HTTPD_USER}
