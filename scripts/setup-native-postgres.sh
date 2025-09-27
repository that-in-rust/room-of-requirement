#!/bin/bash

# Native PostgreSQL Setup Script for macOS
# This script installs and configures PostgreSQL using Homebrew

set -e

echo "🚀 Setting up native PostgreSQL on macOS..."

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This script is for macOS only. Use setup-local-db.sh for Docker setup."
    exit 1
fi

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew is not installed. Installing Homebrew first..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Install PostgreSQL
echo "🐘 Installing PostgreSQL via Homebrew..."
brew install postgresql@15

# Start PostgreSQL service
echo "🚀 Starting PostgreSQL service..."
brew services start postgresql@15

# Wait a moment for PostgreSQL to start
sleep 3

# Create database and user
echo "🔧 Setting up database and user..."

# Create the database
createdb github_pg_query 2>/dev/null || echo "Database might already exist"

# Connect and set up user
psql postgres << EOF
-- Create user if not exists
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'github_user') THEN
        CREATE USER github_user WITH PASSWORD 'secure_password';
    END IF;
END
\$\$;

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE github_pg_query TO github_user;

-- Create extensions
\c github_pg_query
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Test connection
SELECT 'Database setup successful!' as status;
EOF

# Test connection with new user
echo "🔍 Testing database connection..."
if psql -h localhost -U github_user -d github_pg_query -c "SELECT 'Connection successful!' as status;" &>/dev/null; then
    echo "✅ Database connection test passed!"
else
    echo "❌ Database connection test failed"
    exit 1
fi

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

echo ""
echo "🎉 Native PostgreSQL setup complete!"
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
echo "   • Connect to DB: psql -U github_user -d github_pg_query"
echo "   • Start PostgreSQL: brew services start postgresql@15"
echo "   • Stop PostgreSQL: brew services stop postgresql@15"
echo "   • Restart PostgreSQL: brew services restart postgresql@15"
echo ""