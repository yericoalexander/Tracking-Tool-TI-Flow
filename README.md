# Rust Microservice dengan Kafka

Contoh implementasi microservice menggunakan Rust dan Apache Kafka untuk komunikasi asynchronous antar service.

## 📋 Table of Contents

- [Arsitektur](#arsitektur)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Cara Menggunakan Kafka UI](#cara-menggunakan-kafka-ui)
- [Testing API](#testing-api)
- [Configuration](#configuration)
- [Monitoring & Debugging](#monitoring--debugging)
- [Troubleshooting](#troubleshooting)

---

## 🏗️ Arsitektur

### Komponen System

- **API Gateway** (port 3001) - HTTP REST API yang menerima request dan berkomunikasi via Kafka
- **Hash Processor** - Background worker yang mengonsumsi message dari Kafka dan generate SHA256 hash
- **Kafka** (port 9092) - Message broker untuk komunikasi antar service
- **Zookeeper** (port 2181) - Kafka cluster coordination
- **Kafka UI** (port 8080) - Web interface untuk monitoring Kafka

### Flow Diagram

```
Client
  │
  │ POST /hash {"data": "hello"}
  ↓
┌─────────────────┐
│  API Gateway    │ (Port 3001)
│   (Service 1)   │
└────────┬────────┘
         │
         │ Publish message
         ↓
┌─────────────────┐
│     Kafka       │ Topic: hash_requests
│  (Port 9092)    │
└────────┬────────┘
         │
         │ Consume message
         ↓
┌─────────────────┐
│ Hash Processor  │ Generate SHA256
│   (Service 2)   │
└────────┬────────┘
         │
         │ Publish response
         ↓
┌─────────────────┐
│     Kafka       │ Topic: hash_responses
│  (Port 9092)    │
└────────┬────────┘
         │
         │ Consume response
         ↓
┌─────────────────┐
│  API Gateway    │ Return JSON response
│   (Service 1)   │
└────────┬────────┘
         │
         ↓
       Client
```

---

## 🔧 Prerequisites

### 1. Install Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### 2. Install Docker Desktop

**macOS:**
```bash
# Via Homebrew
brew install --cask docker

# Atau download dari: https://www.docker.com/products/docker-desktop
```

**Linux:**
```bash
# Install Docker Engine
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Install Docker Compose
sudo apt-get install docker-compose-plugin
```

### 3. Verify Docker

```bash
docker --version
docker compose version
```

---

## 🚀 Quick Start

### Step 1: Clone & Build

```bash
# Clone repository (atau navigate ke project directory)
cd RUST-KAFKA

# Build project
cargo build
```

### Step 2: Start Kafka Cluster

```bash
# Start Kafka, Zookeeper, dan Kafka UI
make up

# Atau manual:
docker compose up -d

# Verify containers running
docker ps
```

**Expected Output:**
```
CONTAINER ID   IMAGE                             STATUS         PORTS
b56c782a3667   provectuslabs/kafka-ui:latest     Up 2 minutes   0.0.0.0:8080->8080/tcp
81fd2b626933   confluentinc/cp-kafka:7.4.0       Up 2 minutes   0.0.0.0:9092->9092/tcp
66d89560919a   confluentinc/cp-zookeeper:7.4.0   Up 2 minutes   0.0.0.0:2181->2181/tcp
```

### Step 3: Start Services

**Option A: Using Makefile**
```bash
make run-both
```

**Option B: Manual (Recommended for Debugging)**

Terminal 1 - Hash Processor:
```bash
cargo run -p hash-processor
```

Terminal 2 - API Gateway:
```bash
cargo run -p api-gateway
```

**Option C: Background Processes**
```bash
# Start hash-processor di background dengan logging
cargo run -p hash-processor 2>&1 | tee /tmp/hash-processor.log &

# Start api-gateway di background
cargo run -p api-gateway 2>&1 | tee /tmp/api-gateway.log &

# Monitor logs
tail -f /tmp/hash-processor.log
tail -f /tmp/api-gateway.log
```

### Step 4: Verify Services

```bash
# Check if API Gateway is running
curl http://localhost:3001/hash

# Should return: Method Not Allowed (expected, POST required)
```

---

## 🖥️ Cara Menggunakan Kafka UI

### Akses Kafka UI

1. **Buka browser** dan akses:
   ```
   http://localhost:8080
   ```

2. **Kafka UI akan otomatis connect** ke Kafka cluster (configured di docker-compose.yml)

### Konfigurasi Koneksi (Sudah Otomatis)

Kafka UI sudah dikonfigurasi untuk connect ke Kafka broker melalui Docker internal network:

```yaml
# docker-compose.yml
kafka-ui:
  environment:
    KAFKA_CLUSTERS_0_NAME: local
    KAFKA_CLUSTERS_0_BOOTSTRAPSERVERS: kafka:29092
```

### Fitur Kafka UI

#### 1. **Dashboard**
- Overview cluster health
- Broker information
- Topic count dan message rate

#### 2. **Topics**

Navigasi: **Topics** → View topics:
- `hash_requests` - Request dari API Gateway ke Hash Processor
- `hash_responses` - Response dari Hash Processor ke API Gateway
- `__consumer_offsets` - Internal Kafka topic

**Actions:**
- 📊 View messages in topic
- 🔍 Search/filter messages
- 📝 Produce new message (manual testing)
- ⚙️ View topic configuration
- 📈 View metrics

#### 3. **Consumer Groups**

Navigasi: **Consumers** → View active consumers:
- `service_1` - API Gateway consumer group
- `service_2` - Hash Processor consumer group

**View:**
- Consumer lag
- Partition assignment
- Offset information

#### 4. **Messages**

Untuk view messages di topic:
1. Click **Topics** → `hash_requests` atau `hash_responses`
2. Click **Messages** tab
3. Filter by:
   - Time range
   - Partition
   - Offset range
   - Key/Value search

#### 5. **Manual Testing via Kafka UI**

**Produce Message ke hash_requests:**
1. Go to **Topics** → `hash_requests`
2. Click **Produce Message** button
3. Input:
   ```json
   {
     "id": "test-123",
     "data": "test from kafka ui"
   }
   ```
4. Click **Produce**
5. Check `hash_responses` topic untuk response

---

## 🧪 Testing API

### Test 1: Basic Hash Request

```bash
curl -X POST http://localhost:3001/hash \
  -H "Content-Type: application/json" \
  -d '{"data": "hello world"}'
```

**Expected Response:**
```json
{
  "hash": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
  "id": "ccb08b9f-596d-48af-beb3-44a2fdb12a78"
}
```

### Test 2: Multiple Requests

```bash
# Test dengan berbagai data
curl -X POST http://localhost:3001/hash \
  -H "Content-Type: application/json" \
  -d '{"data": "test123"}'

curl -X POST http://localhost:3001/hash \
  -H "Content-Type: application/json" \
  -d '{"data": "microservice"}'

curl -X POST http://localhost:3001/hash \
  -H "Content-Type: application/json" \
  -d '{"data": "rust-kafka-demo"}'
```

### Test 3: Verify di Kafka UI

1. **Buka Kafka UI**: http://localhost:8080
2. **Go to Topics** → `hash_requests`
3. **View Messages** - Anda akan lihat request yang di-publish
4. **Go to Topics** → `hash_responses`
5. **View Messages** - Anda akan lihat response dengan hash result

### Test 4: Load Testing (Optional)

```bash
# Install vegeta (load testing tool)
brew install vegeta  # macOS
# atau apt-get install vegeta  # Linux

# Create test file
echo "POST http://localhost:3001/hash
Content-Type: application/json
@body.json" > targets.txt

echo '{"data": "load test"}' > body.json

# Run load test: 10 requests/second for 10 seconds
vegeta attack -targets=targets.txt -rate=10 -duration=10s | vegeta report
```

### Test 5: Error Cases

**Invalid JSON:**
```bash
curl -X POST http://localhost:3001/hash \
  -H "Content-Type: application/json" \
  -d 'invalid json'
# Expected: 400 Bad Request
```

**Missing data field:**
```bash
curl -X POST http://localhost:3001/hash \
  -H "Content-Type: application/json" \
  -d '{"wrong_field": "value"}'
# Expected: 400 Bad Request
```

---

## ⚙️ Configuration

### Environment Variables

Project menggunakan default values, tapi bisa di-override dengan environment variables:

```bash
# Kafka Configuration
export KAFKA_BROKERS=localhost:9092
export KAFKA_REQUEST_TOPIC=hash_requests
export KAFKA_RESPONSE_TOPIC=hash_responses
export KAFKA_TIMEOUT=5          # seconds
export REQUEST_TIMEOUT=10       # seconds

# API Gateway Configuration
export SERVICE_1_HOST=0.0.0.0
export SERVICE_1_PORT=3001
export SERVICE_1_GROUP_ID=service_1

# Hash Processor Configuration
export SERVICE_2_GROUP_ID=service_2
```

### Development Setup (.env file)

Create `.env` file (optional):
```bash
KAFKA_BROKERS=localhost:9092
KAFKA_REQUEST_TOPIC=hash_requests
KAFKA_RESPONSE_TOPIC=hash_responses
SERVICE_1_PORT=3001
```

---

## 📊 Monitoring & Debugging

### 1. Check Container Logs

```bash
# Kafka logs
docker logs kafka --tail 50 -f

# Zookeeper logs
docker logs zookeeper --tail 50 -f

# Kafka UI logs
docker logs kafka-ui --tail 50 -f
```

### 2. Check Service Logs

```bash
# If running in background with tee
tail -f /tmp/hash-processor.log
tail -f /tmp/api-gateway.log

# Or check cargo output directly in terminal
```

### 3. Kafka Commands

```bash
# List topics
docker exec kafka kafka-topics \
  --bootstrap-server localhost:9092 \
  --list

# Describe topic
docker exec kafka kafka-topics \
  --bootstrap-server localhost:9092 \
  --describe --topic hash_requests

# View consumer groups
docker exec kafka kafka-consumer-groups \
  --bootstrap-server localhost:9092 \
  --list

# Consumer group details
docker exec kafka kafka-consumer-groups \
  --bootstrap-server localhost:9092 \
  --describe --group service_1
```

### 4. Monitor Message Flow

```bash
# Consume messages from hash_requests topic
docker exec kafka kafka-console-consumer \
  --bootstrap-server localhost:9092 \
  --topic hash_requests \
  --from-beginning

# Consume from hash_responses
docker exec kafka kafka-console-consumer \
  --bootstrap-server localhost:9092 \
  --topic hash_responses \
  --from-beginning
```

---

## 🔧 Troubleshooting

### Issue: Kafka tidak start

**Solution:**
```bash
# Stop dan clean
docker compose down -v

# Start ulang
docker compose up -d

# Check logs
docker logs kafka
```

### Issue: API Gateway timeout

**Symptoms:** Request timeout, tidak dapat response

**Possible Causes:**
1. Hash Processor tidak running
2. Kafka broker down
3. Topic tidak tersedia

**Solution:**
```bash
# 1. Verify Kafka running
docker ps | grep kafka

# 2. Verify hash-processor running
ps aux | grep hash-processor

# 3. Check topics exist
docker exec kafka kafka-topics --bootstrap-server localhost:9092 --list

# 4. Restart services
pkill -f "cargo run"
cargo run -p hash-processor &
cargo run -p api-gateway &
```

### Issue: Connection refused

**Solution:**
```bash
# Verify Kafka port
netstat -an | grep 9092

# Verify API Gateway port
netstat -an | grep 3001

# Check if ports are already in use
lsof -i :9092
lsof -i :3001
```

### Issue: Kafka UI tidak connect

**Solution:**
```bash
# Check Kafka UI logs
docker logs kafka-ui

# Verify Kafka accessible from Kafka UI container
docker exec kafka-ui wget -O- kafka:29092

# Restart Kafka UI
docker restart kafka-ui
```

### Issue: Build error

**Solution:**
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Build again
cargo build
```

---

## 🛑 Stop Services

```bash
# Stop background services (if using &)
pkill -f "cargo run"

# Or find and kill process
ps aux | grep cargo
kill <PID>

# Stop Kafka cluster
make down
# atau
docker compose down

# Stop dan remove volumes
docker compose down -v
```

---

## 📚 Additional Resources

- **Rust Documentation**: https://doc.rust-lang.org/
- **Tokio Async Runtime**: https://tokio.rs/
- **Axum Web Framework**: https://github.com/tokio-rs/axum
- **rdkafka Client**: https://docs.rs/rdkafka/
- **Apache Kafka**: https://kafka.apache.org/documentation/
- **Kafka UI**: https://github.com/provectus/kafka-ui

---

## 📝 Architecture Notes

### Request-Response Pattern via Kafka

Project ini menggunakan **correlation ID pattern** untuk request-response via Kafka:

1. API Gateway generate UUID untuk setiap request
2. Store oneshot channel di HashMap dengan UUID sebagai key
3. Publish request dengan UUID ke Kafka
4. Hash Processor consume request, process, dan publish response dengan same UUID
5. API Gateway consume response, match UUID, dan send hasil ke oneshot channel
6. HTTP handler receive hasil dari channel dan return ke client

### Why this Architecture?

- ✅ **Scalability**: Multiple hash processors bisa run parallel
- ✅ **Resilience**: Jika hash processor crash, request bisa di-retry
- ✅ **Decoupling**: Services tidak perlu tahu lokasi satu sama lain
- ✅ **Event-driven**: Mudah add service baru yang subscribe ke Kafka topics
- ✅ **Observability**: Semua message flow visible di Kafka UI

---

## 🎯 Next Steps

Improvement suggestions:
- [ ] Add logging framework (tracing-subscriber)
- [ ] Add metrics (prometheus)
- [ ] Add health check endpoints
- [ ] Add retry mechanism
- [ ] Add dead letter queue for failed messages
- [ ] Add authentication/authorization
- [ ] Add unit tests dan integration tests
- [ ] Add Docker build untuk services
- [ ] Add Kubernetes manifests