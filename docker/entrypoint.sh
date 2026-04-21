#!/usr/bin/env sh
set -eu

: "${BIND_ADDR:=0.0.0.0:3000}"
: "${EBOOKS_ROOT:=/ebooks}"
: "${AUDIOBOOKS_ROOT:=/audiobooks}"
: "${DATABASE_PATH:=/config/book-router.sqlite}"
: "${UMASK:=002}"

umask "$UMASK"

mkdir -p "$(dirname "$DATABASE_PATH")" "$EBOOKS_ROOT" "$AUDIOBOOKS_ROOT"

if [ -n "${PUID:-}" ] || [ -n "${PGID:-}" ]; then
	: "${PUID:?Set both PUID and PGID together}"
	: "${PGID:?Set both PUID and PGID together}"

	group_name="athena"
	if getent group "$PGID" >/dev/null 2>&1; then
		group_name="$(getent group "$PGID" | cut -d: -f1)"
	else
		if getent group "$group_name" >/dev/null 2>&1; then
			group_name="athena-${PGID}"
		fi
		groupadd -o -g "$PGID" "$group_name"
	fi

	if ! getent passwd "$PUID" >/dev/null 2>&1; then
		user_name="athena"
		if getent passwd "$user_name" >/dev/null 2>&1; then
			user_name="athena-${PUID}"
		fi
		useradd -o -u "$PUID" -g "$group_name" -d /app -s /usr/sbin/nologin "$user_name"
	fi

	db_dir="$(dirname "$DATABASE_PATH")"
	chown "$PUID:$PGID" "$db_dir" 2>/dev/null || true

	for db_path in \
		"$DATABASE_PATH" \
		"${DATABASE_PATH}-wal" \
		"${DATABASE_PATH}-shm"; do
		if [ -e "$db_path" ]; then
			chown "$PUID:$PGID" "$db_path" 2>/dev/null || true
			chmod u+rw "$db_path" 2>/dev/null || true
		fi
	done

	exec gosu "$PUID:$PGID" "$@"
fi

exec "$@"
