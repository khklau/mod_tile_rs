FROM httpd:2.4.57-bullseye

ARG HTTPD_USER=www-data
ARG PGDATABASE=osm
ARG PGUSER=osm
ARG PGPASSWORD=osm
ARG RENDER_GROUP_NAME=render
ARG RENDER_GROUP_ID=1111
ENV HTTPD_USER=${HTTPD_USER}
ENV PGDATABASE=${PGDATABASE}
ENV PGUSER=${PGUSER}
ENV PGPASSWORD=${PGPASSWORD}
ENV RENDER_GROUP_NAME=${RENDER_GROUP_NAME}
ENV RENDER_GROUP_ID=${RENDER_GROUP_ID}

# Install the dependencies
RUN apt-get update && \
    apt-get install -y \
        unattended-upgrades \
        renderd=0.5-2 \
        fonts-dejavu \
        fonts-unifont \
        fonts-noto-cjk \
        fonts-noto-cjk-extra \
        fonts-noto-hinted \
        fonts-noto-unhinted

# Stay up to date with security updates
RUN unattended-upgrade

# Setup user and group accounts
RUN adduser --system --group ${PGUSER}
RUN addgroup --gid ${RENDER_GROUP_ID} ${RENDER_GROUP_NAME}
RUN usermod --append --groups ${RENDER_GROUP_NAME} ${PGUSER}

# Add the configuration files
COPY renderd.conf /etc/renderd.conf
COPY style.xml /etc/renderd-style.xml

# Setup directories
RUN mkdir -p /shared/run/renderd
RUN mkdir -p /shared/cache/renderd/tiles
RUN chown -R ${PGUSER}:${RENDER_GROUP_NAME} /shared/run/renderd
RUN chown -R ${PGUSER}:${RENDER_GROUP_NAME} /shared/cache/renderd/tiles

# Execute https as non-root user
USER ${PGUSER}

CMD /usr/bin/renderd --foreground --config /etc/renderd.conf
