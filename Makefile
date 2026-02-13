d ?= 
b ?= bash
s ?= api-gateway

# Docker commands
ps:
	docker compose ps ${d}

up:
	docker compose up -d ${d}

down:
	docker compose down ${d}

log:
	docker compose logs ${d} --tail=100 -f

in:
	docker compose exec ${d} ${b}

restart: down up log

# Rust build commands
build:
	cargo build --release

build-dev:
	cargo build

build-service:
	cargo build --release -p ${s}

# Run commands
run-api:
	cargo run -p api-gateway

run-processor:
	cargo run -p hash-processor

run-both:
	@echo "Starting hash-processor in background..."
	cargo run -p hash-processor &
	@sleep 2
	@echo "Starting api-gateway..."
	cargo run -p api-gateway

# Run binaries
run-bin-api:
	./target/release/api-gateway

run-bin-processor:
	./target/release/hash-processor

# Development workflow
dev: up build-dev run-both

# Production workflow
prod: build run-bin-s1

# Docker build commands
# build-docker:
# 	docker build -f api-gateway/Dockerfile -t api-gateway:latest .
# 	docker build -f hash-processor/Dockerfile -t hash-processor:latest .

# build-docker-api:
# 	docker build -f api-gateway/Dockerfile -t api-gateway:latest .

# build-docker-processor:
# 	docker build -f hash-processor/Dockerfile -t hash-processor:latest .

# Docker run commands
# run-docker:
# 	docker compose -f docker-compose.services.yml up -d

# run-docker-build:
# 	docker compose -f docker-compose.services.yml up --build -d

# stop-docker:
# 	docker compose -f docker-compose.services.yml down

# logs-docker:
# 	docker compose -f docker-compose.services.yml logs -f

# # Test containerized services
# test-docker:
# 	@echo "Waiting for services to start..."
# 	@sleep 15
# 	@make test-api

# Clean
clean:
# Clean
clean:
	cargo clean

clean-docker:
	docker compose -f docker-compose.services.yml down -v
	docker rmi api-gateway:latest hash-processor:latest || true

# Test
test:
	cargo test

# Integration tests
test-api:
	@echo "Testing API endpoint..."
	@echo "Input: 'hello world XXX'"
	@echo "Response:"
	@response=$$(curl -X POST http://localhost:3001/hash \
		-H "Content-Type: application/json" \
		-d '{"data": "hello world XXX"}' \
		-w "###%{http_code}###%{time_total}" \
		-s); \
	json=$$(echo "$$response" | sed 's/###.*//'); \
	status=$$(echo "$$response" | sed 's/.*###\([0-9]*\)###.*/\1/'); \
	time=$$(echo "$$response" | sed 's/.*###[0-9]*###\(.*\)/\1/'); \
	echo "$$json" | jq '.' || echo "$$json"; \
	echo "HTTP Status: $$status"; \
	echo "Response Time: $${time}s"
	@echo "Expected SHA256: b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
	@echo "---"

test-api-multiple:
	@echo "Testing multiple hash requests..."
	@for i in 1 2 3; do \
		echo ""; \
		echo "Test $$i:"; \
		echo "Input: 'test message $$i'"; \
		echo "Response:"; \
		response=$$(curl -X POST http://localhost:3001/hash \
			-H "Content-Type: application/json" \
			-d "{\"data\": \"test message $$i\"}" \
			-w "###%{http_code}###%{time_total}" \
			-s); \
		json=$$(echo "$$response" | sed 's/###.*//'); \
		status=$$(echo "$$response" | sed 's/.*###\([0-9]*\)###.*/\1/'); \
		time=$$(echo "$$response" | sed 's/.*###[0-9]*###\(.*\)/\1/'); \
		echo "$$json" | jq '.' || echo "$$json"; \
		echo "HTTP Status: $$status"; \
		echo "Response Time: $${time}s"; \
		echo "---"; \
	done
# Performance test
test-perf:
	@echo "Performance test - 10 concurrent requests"
	@echo "Input: 'performance test'"
	@for i in {1..10}; do \
		( curl -X POST http://localhost:3001/hash \
			-H "Content-Type: application/json" \
			-d '{"data": "performance test"}' \
			-w "Request $$i - Status: %{http_code} Time: %{time_total}s\n" \
			-s -o /dev/null ) & \
	done; wait
	@echo "All requests completed"

# Simple API test script
test-script:
	./test-api.sh

test-health:
	@echo "Checking if services are running..."
	@if curl -s http://localhost:3001/hash > /dev/null 2>&1; then \
		echo "✅ Service 1 is running"; \
	else \
		echo "❌ Service 1 is not responding"; \
	fi

test-kafka:
	@echo "Checking Kafka topics..."
	docker exec kafka kafka-topics --bootstrap-server localhost:9092 --list

test-full: up
	@echo "Starting full integration test..."
	@echo "Waiting for Kafka to be ready..."
	@sleep 10
	@echo "Starting services in background..."
	@cargo run -p service_2 > /dev/null 2>&1 &
	@sleep 3
	@cargo run -p service_1 > /dev/null 2>&1 &
	@sleep 5
	@echo "Running API tests..."
	@make test-api
	@echo "Stopping services..."
	@pkill -f "service_1|service_2" || true
