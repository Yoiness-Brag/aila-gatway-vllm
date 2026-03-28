#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

if [ -f .env ]; then
    source .env
fi

GATEWAY_URL="http://localhost:${PORT:-3000}"
MANAGEMENT_URL="http://localhost:${MANAGEMENT_PORT:-8080}"
API_BASE="${MANAGEMENT_URL}/api/v1/management"

echo -e "${BLUE}Deleting All Configuration${NC}"
echo "=============================================="

echo -e "${BLUE}1. Checking if gateway is running...${NC}"
if ! curl -s "${MANAGEMENT_URL}/health" > /dev/null; then
    echo -e "${RED}ERROR: Management API is not running at ${MANAGEMENT_URL}${NC}"
    echo -e "${YELLOW}Please start the gateway first:${NC}"
    echo -e "   ${BLUE}HUB_MODE=database cargo run${NC}"
    exit 1
fi
echo -e "${GREEN}   [OK] Gateway is running${NC}"

echo -e "${BLUE}2. Deleting all pipelines...${NC}"
PIPELINES_RESPONSE=$(curl -s "${API_BASE}/pipelines")
PIPELINES_IDS=$(echo "$PIPELINES_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
for PIPELINE_ID in $PIPELINES_IDS; do
    curl -s -X DELETE "${API_BASE}/pipelines/$PIPELINE_ID" > /dev/null
    echo -e "${GREEN}   [OK] Deleted pipeline $PIPELINE_ID${NC}"
done

echo -e "${BLUE}3. Deleting all model definitions...${NC}"
MODEL_DEFINITIONS_RESPONSE=$(curl -s "${API_BASE}/model-definitions")
MODEL_DEFINITIONS_IDS=$(echo "$MODEL_DEFINITIONS_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
for MODEL_DEFINITION_ID in $MODEL_DEFINITIONS_IDS; do
    curl -s -X DELETE "${API_BASE}/model-definitions/$MODEL_DEFINITION_ID" > /dev/null
    echo -e "${GREEN}   [OK] Deleted model $MODEL_DEFINITION_ID${NC}"
done

echo -e "${BLUE}4. Deleting all providers...${NC}"
PROVIDERS_RESPONSE=$(curl -s "${API_BASE}/providers")
PROVIDERS_IDS=$(echo "$PROVIDERS_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
for PROVIDER_ID in $PROVIDERS_IDS; do
    curl -s -X DELETE "${API_BASE}/providers/$PROVIDER_ID" > /dev/null
    echo -e "${GREEN}   [OK] Deleted provider $PROVIDER_ID${NC}"
done

echo -e "${BLUE}5. Deleting all OAuth clients...${NC}"
OAUTH_RESPONSE=$(curl -s "${API_BASE}/oauth-clients")
OAUTH_IDS=$(echo "$OAUTH_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
for OAUTH_ID in $OAUTH_IDS; do
    curl -s -X DELETE "${API_BASE}/oauth-clients/$OAUTH_ID" > /dev/null
    echo -e "${GREEN}   [OK] Deleted OAuth client $OAUTH_ID${NC}"
done

echo ""
echo -e "${GREEN}All configuration deleted successfully!${NC}"
echo ""
echo -e "${GREEN}Happy testing!${NC}"