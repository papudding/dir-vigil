#!/bin/sh
CMD="/usr/local/bin/dir-vigil -d /vigilDir"

[ -n "$TIMEOUT_SECONDS" ] && CMD+=" --timeout_seconds $TIMEOUT_SECONDS"
[ -n "$WARNING_SECONDS" ] && CMD+=" --warning_seconds $WARNING_SECONDS"
[ -n "$CHECKING_INTERVAL" ] && CMD+=" --chinking_interval $CHECKING_INTERVAL"
[ -n "$ALERT_URL" ] && CMD+=" --alert_url $ALERT_URL"
[ -n "$ALERT_CHANNEL" ] && CMD+=" --alert_channel $ALERT_CHANNEL"
[ -n "$COMMENT" ] && CMD+=" --COMMENT $COMMENT"

echo "$CMD"
eval "$CMD"