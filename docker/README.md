# Docker services for Second Brain Router

## Start

```bash
cd docker
docker compose up -d
```

## Services

| Service | Port | URL |
|---------|------|-----|
| qdrant REST + UI | 6333 | http://localhost:6333/dashboard |
| qdrant gRPC | 6334 | used by sbr-daemon |

## Stop

```bash
docker compose down          # keep data
docker compose down -v       # wipe data
```

## Ollama (runs on host, not in Docker)

```bash
brew install ollama
ollama serve &
ollama pull nomic-embed-text
```
