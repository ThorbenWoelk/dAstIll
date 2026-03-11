# dAstIll

Stop doom-scrolling, start deep-diving. dAstIll monitors your favorite YouTube channels, pulls transcripts, and delivers AI-generated summaries - so you can quickly spot what matters to you and spend your time on the videos worth watching. 

daStIll is a full-stack Rust + SvelteKit application that uses local Ollama LLMs to generate and quality-score summaries from transcripts. 
Built with a focus on reliability, performance, and clear architectural separation.

## Features

- **Channel Management**: Tracks favorite YouTube channels, backfills historical videos, and auto-refreshes for new content.
- **AI Summarization**: Generates insightful summaries in a consistent way, evaluated by LLM-as-a-judge for quality of summary related to ground-truth. 
- **Background Workers**: Automatic, asynchronous syncing, downloading, and generating of summaries and evals.

## Tech Stack

### Frontend
- SvelteKit, TypeScript, bun

### Backend
Rust, Turso, Ollama

### Infrastructure & Deployment
Terraform, Google Cloud Run, Google Secret Manager, Artifact Registry, GitHub Actions, Docker

## Prerequisites

To run the application locally, you will need:
- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- [Ollama](https://ollama.com/) (running locally if using local AI models)
- A [Turso](https://turso.tech/) database URL and Auth Token.
- (Optional) YouTube Data API Key.

## Getting Started (Local Development)

1. **Clone the repository**:
   ```bash
   git clone https://github.com/ThorbenWoelk/dAstIll.git
   cd dAstIll
   ```

2. **Configure Environment Variables**:
   Create a `.env` file in the root directory (or in the `backend/` directory) with your database credentials:
   ```env
   DB_URL=libsql://your-turso-db.turso.io
   DB_PASS=your-turso-auth-token
   YOUTUBE_API_KEY=optional-api-key
   OLLAMA_URL=http://localhost:11434
   OLLAMA_MODEL=qwen3:8b
   ```

3. **Start the Application**:
   You can start both the frontend and backend simultaneously using the provided startup script:
   ```bash
   ./start_app.sh
   ```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
