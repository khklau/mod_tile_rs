#!/bin/sh

echo "Executing \"$@\""

chown -R ${RENDER_USER}:${RENDER_GROUP} ${RENDER_BASE_DIR}
cd ${RENDER_BASE_DIR}

exec runuser --user ${RENDER_USER} --group ${RENDER_GROUP} "$@"
