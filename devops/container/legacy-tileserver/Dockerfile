FROM httpd:2.4.57-bullseye

ARG HTTPD_USER=www-data
ARG HTTPD_DIR=/usr/local/apache2/htdocs
ARG RENDER_GROUP_NAME=render
ARG RENDER_GROUP_ID=1111
ENV HTTPD_USER=${HTTPD_USER}
ENV HTTPD_DIR=${HTTPD_DIR}
ENV RENDER_GROUP_NAME=${RENDER_GROUP_NAME}
ENV RENDER_GROUP_ID=${RENDER_GROUP_ID}

# Install the dependencies
RUN apt-get update && \
    apt-get install -y \
        unattended-upgrades \
        wget \
        unzip \
        libapache2-mod-tile=0.5-2 \
        fonts-dejavu \
        fonts-unifont \
        fonts-noto-cjk \
        fonts-noto-cjk-extra \
        fonts-noto-hinted \
        fonts-noto-unhinted

# Stay up to date with security updates
RUN unattended-upgrade

# Setup user and group accounts
RUN addgroup --gid ${RENDER_GROUP_ID} ${RENDER_GROUP_NAME}
RUN usermod --append --groups ${RENDER_GROUP_NAME} ${HTTPD_USER}

# Add the configuration files
COPY legacy-tileserver/tile.conf /etc/apache2/conf-available/tile.conf
COPY legacy-renderer/renderd.conf /etc/renderd.conf

RUN mv /etc/apache2/ports.conf /etc/apache2/ports.orig.conf
COPY legacy-tileserver/ports.conf /etc/apache2/ports.conf

RUN mv /usr/local/apache2/conf/httpd.conf /usr/local/apache2/conf/httpd.orig.conf
COPY legacy-tileserver/httpd.conf /usr/local/apache2/conf/httpd.conf

RUN mv /etc/apache2/sites-available/000-default.conf /etc/apache2/sites-available/000-default.orig.conf
COPY legacy-tileserver/000-default.conf /etc/apache2/sites-available/000-default.conf

# Add the static web resources
COPY legacy-tileserver/index.html ${HTTPD_DIR}/index.html

# Setup directories
WORKDIR ${HTTPD_DIR}
RUN wget http://cdn.leafletjs.com/leaflet/v1.7.1/leaflet.zip
RUN unzip leaflet.zip
RUN rm leaflet.zip
RUN chown -R ${HTTPD_USER} ${HTTPD_DIR}
RUN chown -R ${HTTPD_USER} /usr/local/apache2

# Enable the tileserver site and mod_tile
#RUN a2ensite tileserver_site.conf
RUN a2ensite 000-default.conf
RUN a2enmod tile

# Execute https as non-root user
USER ${HTTPD_USER}
