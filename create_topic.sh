#!/bin/bash

# Wait for Kafka to be ready
echo "Waiting for Kafka to start..."
sleep 10

# Create topic
kafka-topics --create \
  --bootstrap-server kafka:29092 \
  --replication-factor 1 \
  --partitions 1 \
  --topic weather-data \
  --if-not-exists

echo "Topic creation script completed."
