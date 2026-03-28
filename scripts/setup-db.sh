#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

DB_NAME="hid_oauth"
DB_USER="hid_oauth"
DB_PASSWORD="hidoauthpassword"
DB_PORT="5432"
CONTAINER_NAME="hid-oauth-postgres"
GATEWAY_PORT="3000"
MANAGEMENT_PORT="8080"
GATEWAY_URL="http://localhost:${GATEWAY_PORT}"
MANAGEMENT_URL="http://localhost:${MANAGEMENT_PORT}"
API_BASE="${MANAGEMENT_URL}/api/v1/management"

echo -e "${BLUE}Hid-OAuth Gateway - Database Mode Setup${NC}"
echo "=================================================="

if ! command -v docker &> /dev/null; then
    echo -e "${RED}ERROR: Docker is required but not installed.${NC}"
    exit 1
fi

echo -e "${BLUE}1. Setting up PostgreSQL database...${NC}"

if docker ps -a --format 'table {{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "   Stopping existing container..."
    docker stop ${CONTAINER_NAME} >/dev/null 2>&1 || true
    docker rm ${CONTAINER_NAME} >/dev/null 2>&1 || true
fi

echo "   Starting PostgreSQL container..."
docker run --name ${CONTAINER_NAME} \
  -e POSTGRES_DB=${DB_NAME} \
  -e POSTGRES_USER=${DB_USER} \
  -e POSTGRES_PASSWORD=${DB_PASSWORD} \
  -p ${DB_PORT}:5432 \
  -d postgres:15-alpine >/dev/null

echo "   Waiting for PostgreSQL to be ready..."
for i in {1..30}; do
    if docker exec ${CONTAINER_NAME} pg_isready -U ${DB_USER} -d ${DB_NAME} >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

if ! docker exec ${CONTAINER_NAME} pg_isready -U ${DB_USER} -d ${DB_NAME} >/dev/null 2>&1; then
    echo -e "${RED}ERROR: PostgreSQL failed to start within 30 seconds${NC}"
    exit 1
fi

echo -e "${GREEN}   [OK] PostgreSQL is ready${NC}"

echo -e "${BLUE}2. Running database migrations...${NC}"

export DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

if command -v sqlx &> /dev/null; then
    sqlx migrate run
    echo -e "${GREEN}   [OK] Database migrations completed${NC}"
else
    echo -e "${YELLOW}   [WARN] sqlx-cli not found. Run: cargo install sqlx-cli --no-default-features --features postgres${NC}"
fi

echo -e "${BLUE}3. Creating .env file...${NC}"

cat > .env << EOF
DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
HID_OAUTH_MODE=database
REQUIRE_AUTH=true
RUST_LOG=info
PORT=${GATEWAY_PORT}
MANAGEMENT_PORT=${MANAGEMENT_PORT}
DB_POLL_INTERVAL_SECONDS=30

VLLM_API_KEY=YOUR_HF_TOKEN
VLLM_ENDPOINT=http://vllm-inference/v1
VLLM_MODEL=allenai/olmOCR-2-7B-1025-FP8

OPENAI_API_KEY=
AZURE_OPENAI_API_KEY=
ANTHROPIC_API_KEY=
EOF

echo -e "${GREEN}   [OK] .env file created${NC}"

echo ""
echo -e "${GREEN}Setup completed successfully!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Start the gateway:"
echo -e "   ${BLUE}HID_OAUTH_MODE=database cargo run${NC}"
echo ""
echo "2. Or use docker-compose:"
echo -e "   ${BLUE}docker-compose up -d${NC}"
echo ""
echo "3. Verify it's running:"
echo -e "   ${BLUE}curl http://localhost:${GATEWAY_PORT}/health${NC}"
echo -e "   ${BLUE}curl http://localhost:${MANAGEMENT_PORT}/health${NC}"
echo ""
echo "4. Register AILA OCR provider:"
echo -e "   ${BLUE}./scripts/create-sample-config.sh${NC}"
echo ""
echo "5. Create OAuth client:"
echo -e "   ${BLUE}curl -X POST ${API_BASE}/oauth-clients \\${NC}"
echo -e "   ${BLUE}     -H \"Content-Type: application/json\" \\${NC}"
echo -e "   ${BLUE}     -d '{\"client_name\": \"My App\", \"client_email\": \"admin@example.com\"}'${NC}"
echo ""
echo -e "${YELLOW}Useful commands:${NC}"
echo -e "   ${BLUE}docker stop ${CONTAINER_NAME}${NC}"
echo -e "   ${BLUE}docker start ${CONTAINER_NAME}${NC}"
echo -e "   ${BLUE}docker logs ${CONTAINER_NAME}${NC}"
echo -e "   ${BLUE}docker exec -it ${CONTAINER_NAME} psql -U ${DB_USER} -d ${DB_NAME}${NC}"
echo ""
echo -e "${GREEN}Happy coding!${NC}"