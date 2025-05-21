#!/bin/sh
CMD="/usr/local/bin/dir-vigil -d /vigilDir"

[ -n "$TIMEOUT_SECONDS" ] && CMD=$CMD" --timeout-seconds $TIMEOUT_SECONDS"
[ -n "$WARNING_SECONDS" ] && CMD=$CMD" --warning-seconds $WARNING_SECONDS"
[ -n "$CHECKING_INTERVAL" ] && CMD=$CMD" --checking-interval $CHECKING_INTERVAL"
[ -n "$ALERT_URL" ] && CMD=$CMD" --alert-url $ALERT_URL"
[ -n "$ALERT_CHANNEL" ] && CMD=$CMD" --alert-channel $ALERT_CHANNEL"
[ -n "$COMMENT" ] && CMD=$CMD" --comment $COMMENT"

CMD=$CMD" > dir-vigil.log 2>&1"

echo "$CMD"
eval "$CMD"