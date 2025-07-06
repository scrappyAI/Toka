#!/bin/bash

# Toka Docker Environment Setup Script
# This script helps configure the new Docker environment separation

set -e

echo "🚀 Toka Docker Environment Setup"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Check if Docker is installed
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    print_status "Docker and Docker Compose are installed"
}

# Check if environment files exist
check_env_files() {
    if [ ! -f "env.dev" ]; then
        print_error "env.dev file not found"
        exit 1
    fi
    
    if [ ! -f "env.prod" ]; then
        print_error "env.prod file not found"
        exit 1
    fi
    
    if [ ! -f "env.cursor" ]; then
        print_error "env.cursor file not found"
        exit 1
    fi
    
    print_status "Environment template files found"
}

# Setup environment files
setup_env_files() {
    print_info "Setting up environment files..."
    
    # Copy environment files if they don't exist
    if [ ! -f ".env.dev" ]; then
        cp env.dev .env.dev
        print_status "Created .env.dev (development environment)"
        print_warning "Please edit .env.dev with your development configuration"
    else
        print_info ".env.dev already exists"
    fi
    
    if [ ! -f ".env.prod" ]; then
        cp env.prod .env.prod
        print_status "Created .env.prod (production environment)"
        print_warning "IMPORTANT: Please edit .env.prod with your production configuration and secure secrets"
    else
        print_info ".env.prod already exists"
    fi
    
    if [ ! -f ".env.cursor" ]; then
        cp env.cursor .env.cursor
        print_status "Created .env.cursor (cursor environment)"
        print_warning "Please edit .env.cursor with your Cursor configuration"
    else
        print_info ".env.cursor already exists"
    fi
}

# Build Docker images
build_images() {
    print_info "Building Docker images..."
    
    echo "Choose which environment to build:"
    echo "1) Development (Dockerfile.dev)"
    echo "2) Production (Dockerfile.prod)"
    echo "3) Cursor (Dockerfile.cursor)"
    echo "4) All environments"
    echo "5) Skip building"
    
    read -p "Enter your choice (1-5): " choice
    
    case $choice in
        1)
            print_info "Building development image..."
            docker build -f Dockerfile.dev -t toka-dev .
            print_status "Development image built successfully"
            ;;
        2)
            print_info "Building production image..."
            docker build -f Dockerfile.prod -t toka-prod .
            print_status "Production image built successfully"
            ;;
        3)
            print_info "Building cursor image..."
            docker build -f Dockerfile.cursor -t toka-cursor .
            print_status "Cursor image built successfully"
            ;;
        4)
            print_info "Building all images..."
            docker build -f Dockerfile.dev -t toka-dev .
            docker build -f Dockerfile.prod -t toka-prod .
            docker build -f Dockerfile.cursor -t toka-cursor .
            print_status "All images built successfully"
            ;;
        5)
            print_info "Skipping image build"
            ;;
        *)
            print_error "Invalid choice"
            exit 1
            ;;
    esac
}

# Show usage examples
show_usage() {
    echo ""
    echo "📖 Usage Examples"
    echo "================="
    echo ""
    echo "Development Environment:"
    echo "  docker-compose -f docker-compose.dev.yml up -d"
    echo "  docker-compose -f docker-compose.dev.yml logs -f"
    echo "  docker-compose -f docker-compose.dev.yml down"
    echo ""
    echo "Production Environment:"
    echo "  docker-compose -f docker-compose.prod.yml up -d"
    echo "  docker-compose -f docker-compose.prod.yml logs -f"
    echo "  docker-compose -f docker-compose.prod.yml down"
    echo ""
    echo "Cursor Background Agents:"
    echo "  docker-compose -f docker-compose.cursor.yml build"
    echo "  docker-compose -f docker-compose.cursor.yml up cursor-agent"
    echo ""
    echo "📋 Port Mappings:"
    echo "  Development:  Toka(8080), Prometheus(9091), Grafana(3002), Redis(6380)"
    echo "  Production:   Toka(8080), Prometheus(9090), Grafana(3001), PostgreSQL(5432), Redis(6379)"
    echo "  Cursor:       No external ports (managed by Cursor IDE)"
    echo ""
}

# Show next steps
show_next_steps() {
    echo ""
    echo "🎯 Next Steps"
    echo "============="
    echo ""
    echo "1. Edit environment files:"
    echo "   - .env.dev for development configuration"
    echo "   - .env.prod for production configuration (IMPORTANT: change all secrets!)"
    echo "   - .env.cursor for Cursor configuration"
    echo ""
    echo "2. Choose your environment:"
    echo "   - Development: Use docker-compose.dev.yml"
    echo "   - Production: Use docker-compose.prod.yml"
    echo "   - Cursor: Use docker-compose.cursor.yml"
    echo ""
    echo "3. Start your chosen environment"
    echo ""
    echo "4. Read README-Docker-Environments.md for detailed documentation"
    echo ""
}

# Main execution
main() {
    echo ""
    check_docker
    check_env_files
    setup_env_files
    build_images
    show_usage
    show_next_steps
    
    echo ""
    print_status "Setup completed successfully!"
    echo ""
    print_info "For detailed documentation, see README-Docker-Environments.md"
    echo ""
}

# Run main function
main "$@" 