FROM httpd:2.4.57-bullseye

RUN apt-get update && \
    apt-get install -y \
        unattended-upgrades\
        libapache2-mod-tile=0.5-2 \
        renderd=0.5-2 \
        javascript-common=11+nmu1 \
        libjs-leaflet=1.7.1~dfsg-2
# Stay up to date with security updates
RUN unattended-upgrade
