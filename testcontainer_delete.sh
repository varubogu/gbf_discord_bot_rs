#!/bin/env bash
docker rm -f $(docker ps -aq --filter "label=org.testcontainers.managed-by=testcontainers")
