#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
import time


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run non-interactive pi repeatedly with the same prompt."
    )
    parser.add_argument("iterations", type=int, help="number of iterations")
    parser.add_argument("prompt", help="prompt passed to pi")
    parser.add_argument("--provider", default="openai-codex")
    parser.add_argument("--model", default="gpt-5.5")
    parser.add_argument("--reasoning", "--thinking", dest="reasoning", default="xhigh")
    parser.add_argument("--retries", type=int, default=8)
    return parser.parse_args()


def retry_delay_minutes(retry_number: int) -> int:
    return min(2**retry_number, 32)


def run_pi(args: argparse.Namespace) -> int:
    command = [
        "pi",
        "--no-session",
        "-p",
        "--provider",
        args.provider,
        "--model",
        args.model,
        "--thinking",
        args.reasoning,
        args.prompt,
    ]
    return subprocess.run(command).returncode


def main() -> int:
    args = parse_args()

    for iteration in range(1, args.iterations + 1):
        print(f"iteration {iteration}/{args.iterations}", flush=True)

        for attempt in range(args.retries + 1):
            if attempt:
                print(
                    f"iteration {iteration}/{args.iterations}: retry {attempt}/{args.retries}",
                    flush=True,
                )

            returncode = run_pi(args)
            if returncode == 0:
                break

            print(
                f"pi exited with code {returncode} on iteration {iteration}/{args.iterations}",
                file=sys.stderr,
                flush=True,
            )

            if attempt < args.retries:
                delay = retry_delay_minutes(attempt + 1)
                print(f"retrying in {delay} minute(s)", file=sys.stderr, flush=True)
                time.sleep(delay * 60)
            else:
                return returncode

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
