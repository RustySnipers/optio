#!/bin/bash
#
# Generate Test mTLS Certificates for Optio Development
#
# This script generates a complete certificate chain for local development:
# - CA certificate (Certificate Authority)
# - Server certificate (for optio-hub)
# - Client certificate (for optio-agent)
#
# IMPORTANT: These certificates are for DEVELOPMENT ONLY.
# Never use these in production environments.
#

set -euo pipefail

# Configuration
CERT_DIR="${1:-certs}"
DAYS_VALID=365
KEY_SIZE=4096

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Optio mTLS Certificate Generator     ${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check for openssl
if ! command -v openssl &> /dev/null; then
    echo -e "${RED}Error: openssl is not installed${NC}"
    exit 1
fi

# Create certificate directory
mkdir -p "$CERT_DIR"
cd "$CERT_DIR"

echo -e "${YELLOW}Generating certificates in: $(pwd)${NC}"
echo ""

# =============================================================================
# 1. Generate CA (Certificate Authority)
# =============================================================================
echo -e "${GREEN}[1/3] Generating Certificate Authority...${NC}"

# CA private key
openssl genrsa -out ca.key $KEY_SIZE 2>/dev/null

# CA certificate
openssl req -new -x509 -days $DAYS_VALID -key ca.key -out ca.crt \
    -subj "/C=US/ST=State/L=City/O=Optio Dev/OU=Security/CN=Optio Test CA" \
    2>/dev/null

echo "  - ca.key       : CA private key"
echo "  - ca.crt       : CA certificate"
echo ""

# =============================================================================
# 2. Generate Server Certificate (for optio-hub)
# =============================================================================
echo -e "${GREEN}[2/3] Generating Server Certificate...${NC}"

# Server private key
openssl genrsa -out server.key $KEY_SIZE 2>/dev/null

# Server CSR (Certificate Signing Request)
cat > server.cnf << EOF
[req]
default_bits = $KEY_SIZE
prompt = no
default_md = sha256
distinguished_name = dn
req_extensions = req_ext

[dn]
C = US
ST = State
L = City
O = Optio Dev
OU = Hub
CN = optio-hub

[req_ext]
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = optio-hub
DNS.3 = *.local
IP.1 = 127.0.0.1
IP.2 = 0.0.0.0
EOF

openssl req -new -key server.key -out server.csr -config server.cnf 2>/dev/null

# Sign server certificate with CA
cat > server_ext.cnf << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = optio-hub
DNS.3 = *.local
IP.1 = 127.0.0.1
IP.2 = 0.0.0.0
EOF

openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out server.crt -days $DAYS_VALID -extfile server_ext.cnf 2>/dev/null

# Clean up temporary files
rm -f server.csr server.cnf server_ext.cnf

echo "  - server.key   : Server private key"
echo "  - server.crt   : Server certificate"
echo ""

# =============================================================================
# 3. Generate Client Certificate (for optio-agent)
# =============================================================================
echo -e "${GREEN}[3/3] Generating Client Certificate...${NC}"

# Client private key
openssl genrsa -out client.key $KEY_SIZE 2>/dev/null

# Client CSR
cat > client.cnf << EOF
[req]
default_bits = $KEY_SIZE
prompt = no
default_md = sha256
distinguished_name = dn

[dn]
C = US
ST = State
L = City
O = Optio Dev
OU = Agent
CN = optio-agent
EOF

openssl req -new -key client.key -out client.csr -config client.cnf 2>/dev/null

# Sign client certificate with CA
cat > client_ext.cnf << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth
EOF

openssl x509 -req -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out client.crt -days $DAYS_VALID -extfile client_ext.cnf 2>/dev/null

# Clean up temporary files
rm -f client.csr client.cnf client_ext.cnf ca.srl

echo "  - client.key   : Client private key"
echo "  - client.crt   : Client certificate"
echo ""

# =============================================================================
# Summary
# =============================================================================
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Certificates Generated Successfully  ${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Certificate files created in: $(pwd)"
echo ""
echo "Files:"
ls -la *.crt *.key
echo ""
echo -e "${YELLOW}Usage:${NC}"
echo ""
echo "For optio-hub (server):"
echo "  export OPTIO_CERT_PATH=$(pwd)/server.crt"
echo "  export OPTIO_KEY_PATH=$(pwd)/server.key"
echo "  export OPTIO_CA_PATH=$(pwd)/ca.crt"
echo ""
echo "For optio-agent (client):"
echo "  export OPTIO_CERT_PATH=$(pwd)/client.crt"
echo "  export OPTIO_KEY_PATH=$(pwd)/client.key"
echo "  export OPTIO_CA_PATH=$(pwd)/ca.crt"
echo "  export OPTIO_HUB_ADDRESS=https://localhost:50051"
echo ""
echo -e "${RED}WARNING: These certificates are for DEVELOPMENT ONLY.${NC}"
echo -e "${RED}Do not use in production environments.${NC}"
