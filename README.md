# dAstIll

dAstIll watches the channels you follow, summarizes every new video, and prints the summaries as a calm morning paper you read one story at a time.

## Start Here

- [Documentation site](https://dastill-docs.web.app)
- [Disclaimer](docs/disclaimer.md)
- [Local development](docs/operations/local-development.md)
- [Deployment and operations](docs/operations/deployment.md)
- [Architecture](docs/architecture/overview.md)
- [AI models](docs/pipelines/ai-models.md)
- [Reader](docs/features/reader.md)
- [Feature docs](docs/features/summarization.md)
- [Security](docs/security/index.md)

## Local Run

```bash
./scripts/link_shared_env.sh
./start_app.sh
```

The same lifecycle is available through Make:

```bash
make start
make stop
make restart
```

Stop the stack:

```bash
./end_app.sh
```

Detailed setup lives in [docs/operations/local-development.md](docs/operations/local-development.md).

## License

MIT. See [LICENSE](LICENSE).
