FROM debian:bookworm-slim AS whisper-builder

ARG WHISPER_CPP_REF=v1.8.4

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
RUN git clone --depth 1 --branch "${WHISPER_CPP_REF}" https://github.com/ggml-org/whisper.cpp.git .
RUN cmake -B build -DCMAKE_BUILD_TYPE=Release -DWHISPER_BUILD_TESTS=OFF && \
    cmake --build build --config Release --target whisper-cli -j"$(nproc)"
RUN mkdir -p /tmp/whisper-libs && \
    find build -name "*.so*" -exec cp -a {} /tmp/whisper-libs/ \;

FROM debian:bookworm-slim

ARG WHISPER_MODEL_URL=https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    ffmpeg \
    libgomp1 \
    libstdc++6 \
    python3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=whisper-builder /src/build/bin/whisper-cli /usr/local/bin/whisper-cli
COPY --from=whisper-builder /tmp/whisper-libs/ /usr/local/lib/
COPY server.py /app/server.py

RUN ldconfig && whisper-cli --help >/dev/null

RUN mkdir -p /models && \
    curl -L --fail --retry 3 -o /models/ggml-base.en.bin "${WHISPER_MODEL_URL}"

ENV ASR_HOST=0.0.0.0
ENV ASR_PORT=5092
ENV ASR_MODEL_PATH=/models/ggml-base.en.bin
ENV ASR_API_KEY=
ENV ASR_MAX_UPLOAD_BYTES=314572800
ENV ASR_TRANSCRIBE_TIMEOUT_SECS=3600

EXPOSE 5092

CMD ["python3", "/app/server.py"]
