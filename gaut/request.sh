#!/bin/sh
set -eu

if [ "$#" -ne 3 ]; then
    echo "usage: $0 <linux|gaut-os|gaut-child|store|run> <source|name> <request.bin>" >&2
    exit 2
fi

PROFILE_NAME=$1
SOURCE=$2
REQUEST=$3

COMMAND=1
INPUT_KIND=source
case "$PROFILE_NAME" in
    linux) PROFILE=1 ;;
    gaut-os) PROFILE=2 ;;
    gaut-child) PROFILE=3 ;;
    store) PROFILE=3; COMMAND=2 ;;
    run) PROFILE=3; COMMAND=3; INPUT_KIND=name ;;
    *)
        echo "unknown Gaut build profile: $PROFILE_NAME" >&2
        exit 2
        ;;
esac

if [ "$INPUT_KIND" = source ] && [ ! -f "$SOURCE" ] && [ ! -d "$SOURCE" ]; then
    echo "Gaut source not found: $SOURCE" >&2
    exit 2
fi
if [ "$PROFILE_NAME" = store ] && [ ! -f "$SOURCE" ]; then
    echo "A stored Gaut source must be one file: $SOURCE" >&2
    exit 2
fi

REQUEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-request.XXXXXX")
cleanup() {
    /bin/rm -rf "$REQUEST_TEMP"
}
trap cleanup EXIT INT TERM

append_u64() {
    VALUE=$1
    DESTINATION=$2
    O0=$(/usr/bin/printf '%03o' $((VALUE & 255)))
    O1=$(/usr/bin/printf '%03o' $(((VALUE >> 8) & 255)))
    O2=$(/usr/bin/printf '%03o' $(((VALUE >> 16) & 255)))
    O3=$(/usr/bin/printf '%03o' $(((VALUE >> 24) & 255)))
    O4=$(/usr/bin/printf '%03o' $(((VALUE >> 32) & 255)))
    O5=$(/usr/bin/printf '%03o' $(((VALUE >> 40) & 255)))
    O6=$(/usr/bin/printf '%03o' $(((VALUE >> 48) & 255)))
    O7=$(/usr/bin/printf '%03o' $(((VALUE >> 56) & 255)))
    /usr/bin/printf "\\$O0\\$O1\\$O2\\$O3\\$O4\\$O5\\$O6\\$O7" >> "$DESTINATION"
}

BODY="$REQUEST_TEMP/body"
: > "$BODY"
append_u64 "$COMMAND" "$BODY"

if [ "$INPUT_KIND" = name ]; then
    NAME=$SOURCE
    NAME_SIZE=$(/usr/bin/printf '%s' "$NAME" | /usr/bin/wc -c | /usr/bin/tr -d ' ')
    if [ "$NAME_SIZE" -eq 0 ] || [ "$NAME_SIZE" -gt 64 ]; then
        echo "A stored Gaut source name needs between 1 and 64 bytes." >&2
        exit 2
    fi
    append_u64 "$NAME_SIZE" "$BODY"
    /usr/bin/printf '%s' "$NAME" >> "$BODY"
else
    FILES="$REQUEST_TEMP/files"
    if [ -d "$SOURCE" ]; then
        /usr/bin/find "$SOURCE" -type f -name '*.gaut' ! -name '._*' -print \
            | LC_ALL=C /usr/bin/sort > "$FILES"
    else
        /usr/bin/printf '%s\n' "$SOURCE" > "$FILES"
    fi

    UNIT_COUNT=$(/usr/bin/wc -l < "$FILES" | /usr/bin/tr -d ' ')
    if [ "$UNIT_COUNT" -eq 0 ] || [ "$UNIT_COUNT" -gt 64 ]; then
        echo "A Gaut request needs between 1 and 64 source units." >&2
        exit 2
    fi
    if [ "$COMMAND" -eq 2 ] && [ "$UNIT_COUNT" -ne 1 ]; then
        echo "A Gaut store request accepts exactly one source unit." >&2
        exit 2
    fi

    append_u64 "$UNIT_COUNT" "$BODY"
    while IFS= read -r FILE; do
        if [ -d "$SOURCE" ]; then
            RELATIVE=${FILE#"$SOURCE"/}
        else
            RELATIVE=${FILE##*/}
        fi
        NAME=${RELATIVE%.gaut}
        NAME=$(/usr/bin/printf '%s' "$NAME" | /usr/bin/tr '/' '.')
        NAME_SIZE=$(/usr/bin/printf '%s' "$NAME" | /usr/bin/wc -c | /usr/bin/tr -d ' ')
        SOURCE_SIZE=$(/usr/bin/wc -c < "$FILE" | /usr/bin/tr -d ' ')

        append_u64 "$NAME_SIZE" "$BODY"
        append_u64 "$SOURCE_SIZE" "$BODY"
        /usr/bin/printf '%s' "$NAME" >> "$BODY"
        /bin/cat "$FILE" >> "$BODY"
    done < "$FILES"
fi

: > "$REQUEST"
append_u64 "$PROFILE" "$REQUEST"
BODY_SIZE=$(/usr/bin/wc -c < "$BODY" | /usr/bin/tr -d ' ')
append_u64 "$BODY_SIZE" "$REQUEST"
/bin/cat "$BODY" >> "$REQUEST"

REQUEST_SIZE=$(/usr/bin/wc -c < "$REQUEST" | /usr/bin/tr -d ' ')
if [ "$REQUEST_SIZE" -gt 131072 ]; then
    echo "Gaut request exceeds the 131072-byte limit." >&2
    exit 2
fi
