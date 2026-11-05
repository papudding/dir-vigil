#!/bin/sh
# 所有应用日志统一放在 /app/logs 下，便于挂载目录整理。
umask 077
# 启动前归档上一次运行的日志：logs/dir-vigil.log -> logs/dir-vigil-YYYYmmdd-HHMMSS.log
# 归档后以追加方式写入新日志，使容器重启不会丢失历史日志。
LOG_DIR="${LOG_DIR:-/app/logs}"
mkdir -p "$LOG_DIR"
chmod 700 "$LOG_DIR" 2>/dev/null || true
LOG_FILE="$LOG_DIR/dir-vigil.log"

if [ -s "$LOG_FILE" ]; then
  archive_base="$LOG_DIR/dir-vigil-$(date +%Y%m%d-%H%M%S)"
  archive="${archive_base}.log"
  n=1
  while [ -e "$archive" ]; do
    archive="${archive_base}-${n}.log"
    n=$((n + 1))
  done
  mv "$LOG_FILE" "$archive"
  chmod 600 "$archive" 2>/dev/null || true
  echo "Archived previous log to $archive"
fi

touch "$LOG_FILE"
chmod 600 "$LOG_FILE" 2>/dev/null || true

CMD="/usr/local/bin/dir-vigil -d /vigilDir"

[ -n "$TIMEOUT_SECONDS" ] && CMD=$CMD" --timeout-seconds $TIMEOUT_SECONDS"
[ -n "$WARNING_SECONDS" ] && CMD=$CMD" --warning-seconds $WARNING_SECONDS"
[ -n "$CHECKING_INTERVAL" ] && CMD=$CMD" --checking-interval $CHECKING_INTERVAL"
[ -n "$ALERT_URL" ] && CMD=$CMD" --alert-url $ALERT_URL"
[ -n "$ALERT_CHANNEL" ] && CMD=$CMD" --alert-channel $ALERT_CHANNEL"
[ -n "$COMMENT" ] && CMD=$CMD" --comment $COMMENT"
[ -n "$BIND_HOST" ] && CMD=$CMD" --host $BIND_HOST"

CMD=$CMD" >> \"$LOG_FILE\" 2>&1"

# Never echo "$CMD": it may contain the alert URL token.
echo "Starting dir-vigil (logs: $LOG_FILE)"
eval "$CMD"
