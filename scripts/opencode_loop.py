#!/usr/bin/env python3
"""
opencode_loop.py — run opencode in a loop, forking from a session or creating fresh sessions.

Usage examples:

  # Run 5 iterations, each forking from session abc123:
  python opencode_loop.py --use-session abc123 -n 5 "Do the next task from TODO.md"

  # Run 3 iterations with fresh sessions each time:
  python opencode_loop.py -n 3 --model "openai/gpt-5.4" "Implement next item"

  # With thinking and custom agent:
  python opencode_loop.py --use-session abc123 -n 10 --model "openai/gpt-5.4" --variant xhigh --agent build "Next task"
"""

import argparse
import subprocess
import sys
import time


def build_cmd(args: argparse.Namespace, iteration: int, prompt: str) -> list[str]:
    """Build the opencode run command for a single iteration."""
    cmd = ["opencode", "run"]

    # Session forking
    if args.use_session:
        cmd += ["--session", args.use_session, "--fork"]

    # Proxy options
    if args.model:
        cmd += ["--model", args.model]
    if args.variant:
        cmd += ["--variant", args.variant]
    if args.agent:
        cmd += ["--agent", args.agent]
    if args.thinking:
        cmd += ["--thinking"]
    if args.dir:
        cmd += ["--dir", args.dir]

    # Title
    title_prefix = args.title_prefix or "loop"
    cmd += ["--title", f"{title_prefix}-{iteration}"]

    # The prompt itself
    cmd.append(prompt)

    return cmd


def run_iteration(cmd: list[str], iteration: int, total: int, dry_run: bool) -> int:
    """Run a single opencode iteration. Returns the process exit code."""
    header = f"[{iteration}/{total}]"
    print(f"\n{'=' * 60}")
    print(f"{header} Running: {' '.join(cmd)}")
    print(f"{'=' * 60}\n")

    if dry_run:
        print(f"{header} (dry-run, skipping)")
        return 0

    result = subprocess.run(cmd)
    print(f"\n{header} Exit code: {result.returncode}")
    return result.returncode


def main():
    parser = argparse.ArgumentParser(
        description="Run opencode in a loop, optionally forking from a session each time.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    parser.add_argument(
        "prompt",
        help="Prompt to send on each iteration.",
    )
    parser.add_argument(
        "-n", "--iterations",
        type=int,
        default=1,
        help="Number of iterations to run (default: 1).",
    )
    parser.add_argument(
        "--use-session",
        metavar="SESSION_ID",
        help="Fork from this session on each iteration. Without this flag, each iteration creates a fresh session.",
    )

    # Proxy options forwarded to opencode run
    proxy = parser.add_argument_group("opencode proxy options")
    proxy.add_argument("-m", "--model", help="Model in provider/model format (e.g. anthropic/claude-sonnet-4-20250514).")
    proxy.add_argument("--variant", help="Model variant / reasoning effort (e.g. high, max, minimal).")
    proxy.add_argument("--agent", help="Agent to use.")
    proxy.add_argument("--thinking", action="store_true", help="Show thinking blocks.")
    proxy.add_argument("--dir", help="Directory to run in.")

    # Loop control
    ctrl = parser.add_argument_group("loop control")
    ctrl.add_argument("--title-prefix", default="loop", help="Prefix for session titles (default: 'loop'). Title becomes '<prefix>-N'.")
    ctrl.add_argument("--delay", type=float, default=0, help="Delay in seconds between iterations (default: 0).")
    ctrl.add_argument("--stop-on-error", action="store_true", help="Stop the loop if opencode exits with non-zero code.")
    ctrl.add_argument("--dry-run", action="store_true", help="Print commands without executing them.")

    args = parser.parse_args()

    if args.iterations < 1:
        parser.error("--iterations must be >= 1")

    print(f"Starting opencode loop: {args.iterations} iteration(s)")
    if args.use_session:
        print(f"  Forking from session: {args.use_session}")
    else:
        print("  Mode: fresh session per iteration")
    print(f"  Prompt: {args.prompt!r}")
    print()

    for i in range(1, args.iterations + 1):
        cmd = build_cmd(args, i, args.prompt)
        rc = run_iteration(cmd, i, args.iterations, args.dry_run)

        if rc != 0 and args.stop_on_error:
            print(f"\nStopping: iteration {i} exited with code {rc}")
            sys.exit(rc)

        if i < args.iterations and args.delay > 0:
            print(f"Waiting {args.delay}s before next iteration...")
            time.sleep(args.delay)

    print(f"\nLoop complete: {args.iterations} iteration(s) finished.")


if __name__ == "__main__":
    main()
