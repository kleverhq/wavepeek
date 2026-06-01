# Agent Helper Scripts

This directory holds small optional helpers for orchestrating coding-agent runs.

## `pi_loop.py`

Runs non-interactive `pi` repeatedly with the same prompt. This is useful for iterative review/fix loops where each turn should start without a persisted pi session.

```bash
tools/agent/pi_loop.py 5 "review the current diff"
```

Defaults:

- provider: `openai-codex`
- model: `gpt-5.5`
- reasoning/thinking: `xhigh`
- retries per iteration: `8`

Each run uses `pi --no-session -p`. On a non-zero exit, the current iteration is retried with exponential backoff in minutes: `2`, `4`, `8`, `16`, `32`, then capped at `32`. If all retries fail, the loop stops and exits with the last `pi` return code.

Useful overrides:

```bash
tools/agent/pi_loop.py 3 "fix approved FSDB issues" \
  --provider openai-codex \
  --model gpt-5.5 \
  --reasoning xhigh \
  --retries 4
```
