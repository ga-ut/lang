#!/bin/sh
set -eu

if [ "$#" -ne 3 ]; then
    echo "usage: $0 <linux|gaut-os> <source.gaut> <request.bin>" >&2
    exit 2
fi

PROFILE_NAME=$1
SOURCE=$2
REQUEST=$3

case "$PROFILE_NAME" in
    linux) PROFILE=1 ;;
    gaut-os) PROFILE=2 ;;
    *)
        echo "unknown Gaut build profile: $PROFILE_NAME" >&2
        exit 2
        ;;
esac

if [ ! -f "$SOURCE" ]; then
    echo "Gaut source not found: $SOURCE" >&2
    exit 2
fi

SIZE=$(/usr/bin/wc -c < "$SOURCE" | /usr/bin/tr -d ' ')
if [ "$SIZE" -gt 131055 ]; then
    echo "Gaut source exceeds the 131055-byte request limit." >&2
    exit 2
fi

PROFILE_OCTAL=$(/usr/bin/printf '%03o' "$PROFILE")
O0=$(/usr/bin/printf '%03o' $((SIZE & 255)))
O1=$(/usr/bin/printf '%03o' $(((SIZE >> 8) & 255)))
O2=$(/usr/bin/printf '%03o' $(((SIZE >> 16) & 255)))
O3=$(/usr/bin/printf '%03o' $(((SIZE >> 24) & 255)))
O4=$(/usr/bin/printf '%03o' $(((SIZE >> 32) & 255)))
O5=$(/usr/bin/printf '%03o' $(((SIZE >> 40) & 255)))
O6=$(/usr/bin/printf '%03o' $(((SIZE >> 48) & 255)))
O7=$(/usr/bin/printf '%03o' $(((SIZE >> 56) & 255)))

/usr/bin/printf "\\$PROFILE_OCTAL\\000\\000\\000\\000\\000\\000\\000" > "$REQUEST"
/usr/bin/printf "\\$O0\\$O1\\$O2\\$O3\\$O4\\$O5\\$O6\\$O7" >> "$REQUEST"
/bin/dd if="$SOURCE" of="$REQUEST" bs=1 seek=16 conv=notrunc 2>/dev/null
