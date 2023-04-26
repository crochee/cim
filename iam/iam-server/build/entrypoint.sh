#!/bin/sh
# alpine不支持bash
set -ex

# first arg is `-f` or `--some-option`
# or first arg is `something.conf`
if [ "${1#-}" != "$1" ] || [ "${1%.conf}" != "$1" ]; then
	set -- iam-server "$@"
fi

exec "$@"
