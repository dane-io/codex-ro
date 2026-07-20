FROM docker.io/library/node:trixie
RUN apt update
RUN apt install bubblewrap -y
RUN npm install -g @openai/codex

ENTRYPOINT ["codex", "--sandbox", "read-only", "--ask-for-approval", "never", "--config", "web_search=disabled"]