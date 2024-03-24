#!/bin/sh

echo "Creating user '${POSTGRES_USER}'"
createuser ${POSTGRES_USER}

echo "Creating database '${POSTGRES_DB}'"
dropdb ${POSTGRES_DB}
createdb -E UTF8 -O ${POSTGRES_USER} ${POSTGRES_DB}
