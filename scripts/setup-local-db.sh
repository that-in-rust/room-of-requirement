#!/bin/bash

# Local Database Setup Script for GitHub PG Query Tool
# This script sets up a local PostgreSQL database using Docker

set -e

echo "🚀 Setting up local PostgreSQL database..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    echo "   Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose is not available. Please install Docker Compose."
    exit 1
fi

# Stop and remove existing container if it exists
echo "🧹 Cleaning up existing containers..."
docker-compose down -v 2>/dev/null || true

# Start PostgreSQL container
echo "🐘 Starting PostgreSQL container..."
docker-compose up -d

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
timeout=60
counter=0

while ! docker-compose exec -T postgres pg_isready -U github_user -d github_pg_query &>/dev/null; do
    if [ $counter -ge $timeout ]; then
        echo "❌ Timeout waiting for PostgreSQL to start"
        docker-compose logs postgres
        exit 1
    fi
    echo "   Still waiting... ($counter/$timeout seconds)"
    sleep 2
    counter=$((counter + 2))
done

echo "✅ PostgreSQL is ready!"

# Set up environment variables
echo "🔧 Setting up environment variables..."

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    cat > .env << EOF
# Database configuration
DATABASE_URL=postgresql://github_user:secure_password@localhost:5432/github_pg_query

# GitHub token (you need to set this)
GITHUB_TOKEN=your_github_token_here
EOF
    echo "📝 Created .env file - please update GITHUB_TOKEN with your actual token"
else
    echo "📝 .env file already exists"
fi

# Test database connection
echo "🔍 Testing database connection..."
if docker-compose exec -T postgres psql -U github_user -d github_pg_query -c "SELECT 'Connection successful!' as status;" &>/dev/null; then
    echo "✅ Database connection test passed!"
else
    echo "❌ Database connection test failed"
    docker-compose logs postgres
    exit 1
fi

# Show connection info
echo ""
echo "🎉 Local database setup complete!"
echo ""
echo "📋 Connection Details:"
echo "   Host: localhost"
echo "   Port: 5432"
echo "   Database: github_pg_query"
echo "   Username: github_user"
echo "   Password: secure_password"
echo ""
echo "🔗 Database URL:"
echo "   postgresql://github_user:secure_password@localhost:5432/github_pg_query"
echo ""
echo "📝 Next Steps:"
echo "   1. Update GITHUB_TOKEN in .env file with your actual GitHub token"
echo "   2. Run: source .env (or restart your terminal)"
echo "   3. Test the application: cargo run -- \"language:rust\" --dry-run"
echo ""
echo "🛠️  Useful Commands:"
echo "   • Connect to DB: docker-compose exec postgres psql -U github_user -d github_pg_query"
echo "   • View logs: docker-compose logs postgres"
echo "   • Stop DB: docker-compose down"
echo "   • Reset DB: docker-compose down -v && docker-compose up -d"
echo ""