#!/bin/bash

# Database Management Script
# Provides common database operations for the GitHub PG Query tool

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Database connection details
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="github_pg_query"
DB_USER="github_user"
DB_PASS="secure_password"

# Check if using Docker or native PostgreSQL
if docker-compose ps postgres &>/dev/null && [ "$(docker-compose ps -q postgres)" ]; then
    USING_DOCKER=true
    PSQL_CMD="docker-compose exec -T postgres psql -U $DB_USER -d $DB_NAME"
    PSQL_CMD_POSTGRES="docker-compose exec -T postgres psql -U postgres"
else
    USING_DOCKER=false
    PSQL_CMD="psql -h $DB_HOST -U $DB_USER -d $DB_NAME"
    PSQL_CMD_POSTGRES="psql -h $DB_HOST -U postgres"
fi

show_help() {
    echo -e "${BLUE}Database Management Script${NC}"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start       Start the database (Docker only)"
    echo "  stop        Stop the database (Docker only)"
    echo "  restart     Restart the database (Docker only)"
    echo "  status      Show database status"
    echo "  connect     Connect to the database"
    echo "  reset       Reset the database (removes all data)"
    echo "  backup      Create a backup of the database"
    echo "  restore     Restore from a backup file"
    echo "  tables      List all repository tables"
    echo "  history     Show query history"
    echo "  stats       Show database statistics"
    echo "  cleanup     Remove old repository tables"
    echo "  help        Show this help message"
    echo ""
}

start_db() {
    if [ "$USING_DOCKER" = true ]; then
        echo -e "${BLUE}Starting PostgreSQL container...${NC}"
        docker-compose up -d postgres
        echo -e "${GREEN}✅ Database started${NC}"
    else
        echo -e "${YELLOW}⚠️  Using native PostgreSQL - use system commands to start${NC}"
        echo "   macOS: brew services start postgresql@15"
        echo "   Linux: sudo systemctl start postgresql"
    fi
}

stop_db() {
    if [ "$USING_DOCKER" = true ]; then
        echo -e "${BLUE}Stopping PostgreSQL container...${NC}"
        docker-compose stop postgres
        echo -e "${GREEN}✅ Database stopped${NC}"
    else
        echo -e "${YELLOW}⚠️  Using native PostgreSQL - use system commands to stop${NC}"
        echo "   macOS: brew services stop postgresql@15"
        echo "   Linux: sudo systemctl stop postgresql"
    fi
}

restart_db() {
    if [ "$USING_DOCKER" = true ]; then
        echo -e "${BLUE}Restarting PostgreSQL container...${NC}"
        docker-compose restart postgres
        echo -e "${GREEN}✅ Database restarted${NC}"
    else
        echo -e "${YELLOW}⚠️  Using native PostgreSQL - use system commands to restart${NC}"
        echo "   macOS: brew services restart postgresql@15"
        echo "   Linux: sudo systemctl restart postgresql"
    fi
}

show_status() {
    echo -e "${BLUE}Database Status:${NC}"
    
    if [ "$USING_DOCKER" = true ]; then
        echo "  Mode: Docker"
        if docker-compose ps postgres | grep -q "Up"; then
            echo -e "  Status: ${GREEN}Running${NC}"
        else
            echo -e "  Status: ${RED}Stopped${NC}"
        fi
    else
        echo "  Mode: Native PostgreSQL"
        if pg_isready -h $DB_HOST -p $DB_PORT &>/dev/null; then
            echo -e "  Status: ${GREEN}Running${NC}"
        else
            echo -e "  Status: ${RED}Not responding${NC}"
        fi
    fi
    
    # Test connection
    if $PSQL_CMD -c "SELECT 1;" &>/dev/null; then
        echo -e "  Connection: ${GREEN}OK${NC}"
    else
        echo -e "  Connection: ${RED}Failed${NC}"
    fi
}

connect_db() {
    echo -e "${BLUE}Connecting to database...${NC}"
    if [ "$USING_DOCKER" = true ]; then
        docker-compose exec postgres psql -U $DB_USER -d $DB_NAME
    else
        psql -h $DB_HOST -U $DB_USER -d $DB_NAME
    fi
}

reset_db() {
    echo -e "${RED}⚠️  This will delete ALL data in the database!${NC}"
    read -p "Are you sure? Type 'yes' to continue: " confirm
    
    if [ "$confirm" = "yes" ]; then
        echo -e "${BLUE}Resetting database...${NC}"
        
        # Drop and recreate database
        $PSQL_CMD_POSTGRES -c "DROP DATABASE IF EXISTS $DB_NAME;"
        $PSQL_CMD_POSTGRES -c "CREATE DATABASE $DB_NAME;"
        $PSQL_CMD_POSTGRES -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;"
        
        # Recreate extensions
        $PSQL_CMD -c "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";"
        
        echo -e "${GREEN}✅ Database reset complete${NC}"
    else
        echo -e "${YELLOW}Reset cancelled${NC}"
    fi
}

backup_db() {
    BACKUP_FILE="backup_$(date +%Y%m%d_%H%M%S).sql"
    echo -e "${BLUE}Creating backup: $BACKUP_FILE${NC}"
    
    if [ "$USING_DOCKER" = true ]; then
        docker-compose exec -T postgres pg_dump -U $DB_USER -d $DB_NAME > "$BACKUP_FILE"
    else
        pg_dump -h $DB_HOST -U $DB_USER -d $DB_NAME > "$BACKUP_FILE"
    fi
    
    echo -e "${GREEN}✅ Backup created: $BACKUP_FILE${NC}"
}

restore_db() {
    if [ -z "$2" ]; then
        echo -e "${RED}❌ Please specify backup file${NC}"
        echo "Usage: $0 restore <backup_file.sql>"
        exit 1
    fi
    
    BACKUP_FILE="$2"
    if [ ! -f "$BACKUP_FILE" ]; then
        echo -e "${RED}❌ Backup file not found: $BACKUP_FILE${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}Restoring from: $BACKUP_FILE${NC}"
    
    if [ "$USING_DOCKER" = true ]; then
        cat "$BACKUP_FILE" | docker-compose exec -T postgres psql -U $DB_USER -d $DB_NAME
    else
        psql -h $DB_HOST -U $DB_USER -d $DB_NAME < "$BACKUP_FILE"
    fi
    
    echo -e "${GREEN}✅ Restore complete${NC}"
}

list_tables() {
    echo -e "${BLUE}Repository Tables:${NC}"
    $PSQL_CMD -c "
        SELECT 
            table_name,
            pg_size_pretty(pg_total_relation_size(table_name::regclass)) as size
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name LIKE 'repos_%'
        ORDER BY table_name DESC;
    "
}

show_history() {
    echo -e "${BLUE}Query History (Last 10):${NC}"
    $PSQL_CMD -c "
        SELECT 
            executed_at,
            search_query,
            table_name,
            result_count,
            duration_ms || 'ms' as duration,
            CASE WHEN success THEN '✅' ELSE '❌' END as status
        FROM query_history 
        ORDER BY executed_at DESC 
        LIMIT 10;
    "
}

show_stats() {
    echo -e "${BLUE}Database Statistics:${NC}"
    
    # Overall stats
    $PSQL_CMD -c "
        SELECT 
            COUNT(*) as total_tables,
            pg_size_pretty(pg_database_size('$DB_NAME')) as database_size
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name LIKE 'repos_%';
    "
    
    echo ""
    echo -e "${BLUE}Query History Summary:${NC}"
    $PSQL_CMD -c "
        SELECT 
            COUNT(*) as total_queries,
            COUNT(*) FILTER (WHERE success) as successful_queries,
            COUNT(*) FILTER (WHERE NOT success) as failed_queries,
            AVG(duration_ms) as avg_duration_ms,
            SUM(result_count) as total_repositories_fetched
        FROM query_history;
    "
}

cleanup_tables() {
    echo -e "${BLUE}Finding old repository tables...${NC}"
    
    # Show tables older than 7 days
    $PSQL_CMD -c "
        SELECT table_name
        FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name LIKE 'repos_%'
        AND table_name < 'repos_' || to_char(NOW() - INTERVAL '7 days', 'YYYYMMDDHH24MISS')
        ORDER BY table_name;
    " -t | while read -r table; do
        if [ -n "$table" ]; then
            table=$(echo "$table" | xargs) # trim whitespace
            echo -e "${YELLOW}Found old table: $table${NC}"
            read -p "Delete table $table? (y/N): " confirm
            if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
                $PSQL_CMD -c "DROP TABLE IF EXISTS $table;"
                echo -e "${GREEN}✅ Deleted $table${NC}"
            fi
        fi
    done
}

# Main command handling
case "${1:-help}" in
    start)
        start_db
        ;;
    stop)
        stop_db
        ;;
    restart)
        restart_db
        ;;
    status)
        show_status
        ;;
    connect)
        connect_db
        ;;
    reset)
        reset_db
        ;;
    backup)
        backup_db
        ;;
    restore)
        restore_db "$@"
        ;;
    tables)
        list_tables
        ;;
    history)
        show_history
        ;;
    stats)
        show_stats
        ;;
    cleanup)
        cleanup_tables
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac