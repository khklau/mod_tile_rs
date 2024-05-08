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
        aptitude \
        libapache2-mod-tile=0.5-2 \
        renderd=0.5-2 \
        javascript-common=11+nmu1 \
        libjs-leaflet=1.7.1~dfsg-2 \
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

# Setup directories
RUN mkdir -p /etc/systemd/system/renderd.service.d/
RUN mkdir -p /run/renderd
RUN mkdir -p /var/cache/renderd/tiles
RUN chown -R osm /run/renderd
RUN chown -R osm /var/cache/renderd/tiles

# Add the configuration files
COPY renderd.conf /etc/renderd.conf
COPY style.xml /etc/renderd-style.xml
COPY httpd.conf /usr/local/apache2/conf/httpd.conf
COPY tile.conf /etc/apache2/mods-available/tile.conf
COPY custom.conf /etc/systemd/system/renderd.service.d/custom.conf

# This is actually the default tile.conf file, removing to avoid confusing
# it with /etc/renderd.conf which is a different configuration file
#RUN rm /etc/apache2/mods-available/renderd.conf

# Enable mod_tile
RUN a2enmod tile

# Run the renderd & webserver
CMD systemctl daemon-reload \
    && systemctl restart renderd apache2
