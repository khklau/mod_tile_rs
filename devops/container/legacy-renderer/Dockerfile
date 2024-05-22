FROM httpd:2.4.57-bullseye

ARG PGDATABASE=osm
ARG PGUSER=osm
ARG PGPASSWORD=osm
ARG RENDER_GROUP=render
ARG RENDER_GROUP_ID=1111
ARG RENDER_BASE_DIR=/shared
ENV HTTPD_USER=${HTTPD_USER}
ENV PGDATABASE=${PGDATABASE}
ENV PGUSER=${PGUSER}
ENV PGPASSWORD=${PGPASSWORD}
ENV RENDER_USER=${PGUSER}
ENV RENDER_GROUP=${RENDER_GROUP}
ENV RENDER_GROUP_ID=${RENDER_GROUP_ID}
ENV RENDER_BASE_DIR=${RENDER_BASE_DIR}

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
RUN adduser --system --group ${RENDER_USER}
RUN addgroup --gid ${RENDER_GROUP_ID} ${RENDER_GROUP}
RUN usermod --append --groups ${RENDER_GROUP} ${RENDER_USER}

# Add the configuration files
COPY renderd.conf /etc/renderd.conf
COPY style.xml /etc/style.xml
COPY entrypoint.sh /entrypoint.sh
COPY exec_renderd.sh /exec_renderd.sh

# Setup directories
VOLUME ${RENDER_BASE_DIR}

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/exec_renderd.sh"]
