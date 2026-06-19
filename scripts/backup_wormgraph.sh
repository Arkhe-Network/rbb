#!/bin/bash
# backup_wormgraph.sh
# Script to automate taking pg_dump backups of the WormGraph database from the
# cathedral-postgres container, compressing them, and cleaning up old backups.

set -e

# Configuration
BACKUP_DIR="/mnt/persist/backups/postgres"
CONTAINER_NAME="cathedral-postgres"
DB_USER="cathedral"
DB_NAME="cathedral"
RETENTION_DAYS=30
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/wormgraph_backup_${TIMESTAMP}.sql"
COMPRESSED_FILE="${BACKUP_FILE}.gz"

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

echo "[$(date)] Starting WormGraph backup..."

# Execute pg_dump inside the postgres container
if docker exec "$CONTAINER_NAME" pg_dump -U "$DB_USER" "$DB_NAME" > "$BACKUP_FILE"; then
    echo "[$(date)] Backup created successfully: $BACKUP_FILE"

    # Compress the backup
    gzip "$BACKUP_FILE"
    echo "[$(date)] Backup compressed to: $COMPRESSED_FILE"
else
    echo "[$(date)] Error: Backup failed." >&2
    exit 1
fi

# Cleanup old backups
echo "[$(date)] Cleaning up backups older than $RETENTION_DAYS days..."
find "$BACKUP_DIR" -type f -name "wormgraph_backup_*.sql.gz" -mtime +$RETENTION_DAYS -exec rm -f {} \;

echo "[$(date)] Backup process completed successfully."
