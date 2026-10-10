import json
import os
import pty
import signal
import subprocess
import tempfile
import time
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DEV = REPO_ROOT / "dev"
GIT_LOCAL_ENV = (
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_PREFIX",
    "GIT_WORK_TREE",
)


class DevTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.tmp = Path(self.temp.name)
        self.main = self.tmp / "main"
        self.linked = self.tmp / "linked"
        self.fake_bin = self.tmp / "bin"
        self.log = self.tmp / "calls.jsonl"
        self.fake_bin.mkdir()
        self._write_fakes()
        self._init_repositories()

    def _base_env(self) -> dict[str, str]:
        env = os.environ.copy()
        for name in GIT_LOCAL_ENV:
            env.pop(name, None)
        env.pop("VERDI_HOME", None)
        env.pop("WAVEPEEK_DEV_CONFIG", None)
        env.pop("WAVEPEEK_FSDB_ABI", None)
        env.pop("WAVEPEEK_FSDB_READER_LIBDIR", None)
        env.pop("WAVEPEEK_FSDB_EMBED_RPATH", None)
        env["PATH"] = f"{self.fake_bin}{os.pathsep}{env['PATH']}"
        env["FAKE_LOG"] = str(self.log)
        return env

    def _write_fakes(self) -> None:
        docker = self.fake_bin / "docker"
        docker.write_text(
            """#!/usr/bin/env python3
import hashlib, json, os, pathlib, signal, sys
with open(os.environ["FAKE_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps(["docker", *sys.argv[1:]]) + "\\n")
args = sys.argv[1:]
if args[:1] == ["ps"]:
    existing_config = os.environ.get("FAKE_EXISTING_CONFIG")
    matches = not existing_config or f"label=dev.wavepeek.config={existing_config}" in args
    if "-aq" in args and os.environ.get("FAKE_EXISTING") == "1" and matches:
        print("container-1")
    elif "-q" in args and not (any(value == "id=container-1" for value in args) and os.environ.get("FAKE_STOPPED") == "1"):
        print("container-1")
elif args[:1] == ["inspect"]:
    if "dev.wavepeek.config_hash" in args[args.index("--format") + 1]:
        config = pathlib.Path(os.environ["FAKE_ROOT"]) / os.environ.get("WAVEPEEK_DEV_CONFIG", ".devcontainer/devcontainer.json")
        print(os.environ.get("FAKE_CONFIG_HASH", hashlib.sha256(config.read_bytes()).hexdigest()))
    else:
        for source, target in json.loads(os.environ.get("FAKE_MOUNTS", "[]")):
            print(f"{source}\\t{target}")
elif args[:3] == ["rm", "-f", "container-1"]:
    pass
elif args[:2] == ["exec", "container-1"]:
    if args[2] in {"cat", "rm"}:
        command = args[2:]
        for index, value in enumerate(command):
            if value.startswith("/tmp/wavepeek-dev-"):
                command[index] = os.environ["FAKE_ROOT"] + "/" + pathlib.Path(value).name
        os.execvp(command[0], command)
    if args[2:4] == ["bash", "-c"]:
        command = args[2:]
        for index, value in enumerate(command):
            if value.startswith("/tmp/wavepeek-dev-"):
                command[index] = os.environ["FAKE_ROOT"] + "/" + pathlib.Path(value).name
        os.execvp(command[0], command)
    raise SystemExit(2)
else:
    raise SystemExit(2)
"""
        )
        docker.chmod(0o755)

        devcontainer = self.fake_bin / "devcontainer"
        devcontainer.write_text(
            """#!/usr/bin/env python3
import json, os, pathlib, sys
with open(os.environ["FAKE_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps(["devcontainer", *sys.argv[1:]]) + "\\n")
args = sys.argv[1:]
if args[:1] == ["up"]:
    raise SystemExit(int(os.environ.get("FAKE_UP_STATUS", "0")))
if args[:1] != ["exec"]:
    raise SystemExit(2)
command = args[args.index("--") + 1:]
workspace = f"/workspaces/{pathlib.Path(os.environ['FAKE_ROOT']).name}"
for index, value in enumerate(command):
    if value == workspace or value.startswith(workspace + "/"):
        command[index] = os.environ["FAKE_ROOT"] + value[len(workspace):]
    elif value.startswith("/tmp/wavepeek-dev-"):
        command[index] = os.environ["FAKE_ROOT"] + "/" + pathlib.Path(value).name
    elif value.startswith("/workspaces/") and value.endswith("/tools/repo/dev_exec.py"):
        command[index] = os.environ["FAKE_ROOT"] + "/tools/repo/dev_exec.py"
os.execvp(command[0], command)
"""
        )
        devcontainer.chmod(0o755)

    def _git(self, *args: str, cwd: Path | None = None) -> None:
        subprocess.run(
            ["git", *args],
            cwd=cwd or self.main,
            env=self._base_env(),
            check=True,
            stdout=subprocess.DEVNULL,
        )

    def _init_repositories(self) -> None:
        self.main.mkdir()
        self._git("init", "-q", "-b", "main")
        self._git("config", "user.name", "Test User")
        self._git("config", "user.email", "test@example.com")
        (self.main / "tracked").write_text("test\n")
        self._git("add", "tracked")
        self._git("commit", "-qm", "test")
        self._git("worktree", "add", "-qb", "linked", str(self.linked))

        for root in (self.main, self.linked):
            (root / ".devcontainer").mkdir()
            (root / ".devcontainer" / "devcontainer.json").write_text("{}\n")
            helper = root / "tools" / "repo" / "dev_exec.py"
            helper.parent.mkdir(parents=True)
            helper.write_bytes((REPO_ROOT / "tools/repo/dev_exec.py").read_bytes())

    def _run(
        self,
        root: Path,
        *args: str,
        cwd: Path | None = None,
        env_updates: dict[str, str] | None = None,
        input_text: str | None = None,
    ) -> subprocess.CompletedProcess[str]:
        env = self._base_env()
        env["FAKE_ROOT"] = str(root)
        if env_updates:
            env.update(env_updates)
        return subprocess.run(
            [str(DEV), *args],
            cwd=cwd or root,
            env=env,
            input=input_text,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def _calls(self) -> list[list[str]]:
        if not self.log.exists():
            return []
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def test_preserves_nested_directory_arguments_io_and_exit_status(self) -> None:
        nested = self.main / "one" / "two"
        nested.mkdir(parents=True)
        code = (
            "import os,sys; data=sys.stdin.read(); "
            "print(os.getcwd()); print(repr(sys.argv[1:])); print(data, end=''); "
            "print('stderr', file=sys.stderr); raise SystemExit(23)"
        )

        result = self._run(
            self.main,
            "python3",
            "-c",
            code,
            "two words",
            "*",
            "",
            cwd=nested,
            input_text="stdin\n",
        )

        self.assertEqual(result.returncode, 23)
        self.assertEqual(
            result.stdout,
            f"{nested}\n['two words', '*', '']\nstdin\n",
        )
        self.assertEqual(result.stderr, "stderr\n")
        calls = self._calls()
        up = next(call for call in calls if call[:2] == ["devcontainer", "up"])
        self.assertEqual(up[up.index("--workspace-folder") + 1], str(self.main))
        execute = next(call for call in calls if call[:2] == ["devcontainer", "exec"])
        self.assertEqual(execute[execute.index("--workspace-folder") + 1], str(self.main))
        self.assertIn("/workspaces/main/one/two", execute)

    def test_main_and_linked_worktrees_have_distinct_identity_and_git_mount(self) -> None:
        for root in (self.main, self.linked):
            self.log.unlink(missing_ok=True)
            result = self._run(root, "true")
            self.assertEqual(result.returncode, 0, result.stderr)
            up = next(call for call in self._calls() if call[:2] == ["devcontainer", "up"])
            self.assertEqual(up[up.index("--workspace-folder") + 1], str(root))
            mount_values = [
                up[index + 1] for index, value in enumerate(up) if value == "--mount"
            ]
            if root == self.main:
                self.assertEqual(mount_values, [])
            else:
                common = str((self.main / ".git").resolve())
                self.assertEqual(
                    mount_values,
                    [f"type=bind,source={common},target={common}"],
                )

    def test_public_profile_ignores_host_sdk_environment(self) -> None:
        result = self._run(
            self.main,
            "true",
            env_updates={
                "VERDI_HOME": str(self.tmp / "missing-verdi"),
                "WAVEPEEK_FSDB_ABI": "unsupported",
                "WAVEPEEK_FSDB_READER_LIBDIR": "/missing",
                "WAVEPEEK_FSDB_EMBED_RPATH": "0",
            },
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        up = next(call for call in self._calls() if call[:2] == ["devcontainer", "up"])
        self.assertNotIn("--mount", up)
        self.assertNotIn("--remote-env", up)
        self.assertEqual(up[up.index("--config") + 1], str(self.main / ".devcontainer/devcontainer.json"))

    def _private_profile(self, root: Path) -> Path:
        config = root / ".devcontainer.local/devcontainer.json"
        config.parent.mkdir()
        config.write_text('{"image": "private-tools", "mounts": ["type=bind,source=/sdk,target=/tools"]}\n')
        return config

    def test_dotenv_selection_and_environment_override_from_nested_directory(self) -> None:
        private = self._private_profile(self.main)
        (self.main / ".env").write_text("WAVEPEEK_DEV_CONFIG=.devcontainer.local/devcontainer.json\n")
        nested = self.main / "nested"
        nested.mkdir()
        for override, expected in (({}, private), ({"WAVEPEEK_DEV_CONFIG": ".devcontainer/devcontainer.json"}, self.main / ".devcontainer/devcontainer.json")):
            with self.subTest(override=override):
                self.log.unlink(missing_ok=True)
                result = self._run(self.main, "true", cwd=nested, env_updates=override)
                self.assertEqual(result.returncode, 0, result.stderr)
                for call in self._calls():
                    if call[0] == "devcontainer":
                        self.assertEqual(call[call.index("--config") + 1], str(expected))
                    elif call[:2] == ["docker", "ps"] and not any(value.startswith("id=") for value in call):
                        self.assertIn(f"label=dev.wavepeek.config={expected}", call)

    def test_private_profile_accepts_its_own_mounts_and_requires_git_mount(self) -> None:
        self._private_profile(self.linked)
        common = str((self.main / ".git").resolve())
        mounts = [[str(self.linked), "/workspaces/linked"], ["/sdk", "/tools"]]
        env = {
            "WAVEPEEK_DEV_CONFIG": ".devcontainer.local/devcontainer.json",
            "FAKE_EXISTING": "1",
            "FAKE_MOUNTS": json.dumps([*mounts, [common, common]]),
        }
        result = self._run(self.linked, "--exec-only", "true", env_updates=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(any(call[:2] == ["devcontainer", "up"] for call in self._calls()))

        self.log.unlink()
        env["FAKE_MOUNTS"] = json.dumps(mounts)
        result = self._run(self.linked, "--exec-only", "true", env_updates=env)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("stale configuration or worktree/Git mounts", result.stderr)
        self.assertFalse(any(call[0] == "devcontainer" for call in self._calls()))

    def test_exec_only_does_not_select_another_profile_container(self) -> None:
        private = self._private_profile(self.main)
        result = self._run(self.main, "--exec-only", "true", env_updates={
            "FAKE_EXISTING": "1", "FAKE_EXISTING_CONFIG": str(private),
        })
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no existing container", result.stderr)
        self.assertFalse(any(call[0] == "devcontainer" for call in self._calls()))

    def test_rejects_missing_absolute_and_escaping_configuration(self) -> None:
        outside = self.tmp / "outside.json"
        outside.write_text("{}\n")
        (self.main / "outside.json").symlink_to(outside)
        for value, message in (
            ("missing.json", "devcontainer config not found"),
            (str(outside), "must be relative"),
            ("../outside.json", "escapes the worktree root"),
            ("outside.json", "escapes the worktree root"),
        ):
            with self.subTest(config=value):
                result = self._run(self.main, "true", env_updates={"WAVEPEEK_DEV_CONFIG": value})
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(message, result.stderr)
                self.assertEqual(self._calls(), [])

    def test_stale_configuration_requires_explicit_profile_recreation(self) -> None:
        private = self._private_profile(self.main)
        env = {
            "WAVEPEEK_DEV_CONFIG": ".devcontainer.local/devcontainer.json",
            "FAKE_EXISTING": "1", "FAKE_CONFIG_HASH": "old-config",
            "FAKE_MOUNTS": json.dumps([[str(self.main), "/workspaces/main"]]),
        }
        for mode in ([], ["--exec-only"]):
            result = self._run(self.main, *mode, "true", env_updates=env)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("./dev --recreate true", result.stderr)
            self.assertIn("WAVEPEEK_DEV_CONFIG=.devcontainer.local/devcontainer.json", result.stderr)
            self.assertFalse(any(call[0] == "devcontainer" for call in self._calls()))

        result = self._run(self.main, "--recreate", "true", env_updates=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        up = next(call for call in self._calls() if call[:2] == ["devcontainer", "up"])
        self.assertIn(f"dev.wavepeek.config={private}", up)
        removals = [call for call in self._calls() if call[:2] == ["docker", "rm"]]
        self.assertEqual(removals, [["docker", "rm", "-f", "container-1"]])

    def test_stale_mounts_are_rejected_with_recreation_command(self) -> None:
        env = {
            "FAKE_EXISTING": "1",
            "FAKE_MOUNTS": json.dumps([[str(self.main), "/wrong-workspace"]]),
        }
        result = self._run(self.main, "true", env_updates=env)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("stale configuration or worktree/Git mounts", result.stderr)
        self.assertIn("./dev --recreate", result.stderr)
        self.assertFalse(any(call[0] == "devcontainer" for call in self._calls()))

    def test_public_profile_rejects_extra_sdk_mount(self) -> None:
        result = self._run(self.main, "--exec-only", "true", env_updates={
            "FAKE_EXISTING": "1",
            "FAKE_MOUNTS": json.dumps([[str(self.main), "/workspaces/main"], ["/sdk", "/opt/verdi"]]),
        })
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("stale configuration or worktree/Git mounts", result.stderr)
        self.assertFalse(any(call[0] == "devcontainer" for call in self._calls()))

    def test_exec_only_uses_existing_container_without_up(self) -> None:
        common = str((self.main / ".git").resolve())
        env = {
            "FAKE_EXISTING": "1",
            "FAKE_MOUNTS": json.dumps(
                [
                    [str(self.linked), "/workspaces/linked"],
                    [common, common],
                ]
            ),
        }
        result = self._run(self.linked, "--exec-only", "printf", "ok", env_updates=env)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "ok")
        devcontainer_calls = [call for call in self._calls() if call[0] == "devcontainer"]
        self.assertEqual(len(devcontainer_calls), 1)
        self.assertEqual(devcontainer_calls[0][:2], ["devcontainer", "exec"])

        self.log.unlink()
        result = self._run(self.main, "--exec-only", "true")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no existing container", result.stderr)
        self.assertIn("./dev true", result.stderr)
        self.assertFalse(any(call[0] == "devcontainer" for call in self._calls()))

        self.log.unlink()
        result = self._run(self.linked, "--exec-only", "true", env_updates=env | {"FAKE_STOPPED": "1"})
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("container is stopped", result.stderr)
        self.assertFalse(any(call[0] == "devcontainer" for call in self._calls()))

    def test_interactive_tty_is_preserved(self) -> None:
        master, slave = pty.openpty()
        env = self._base_env()
        env["FAKE_ROOT"] = str(self.main)
        process = subprocess.Popen(
            [
                str(DEV),
                "python3",
                "-c",
                "import os,sys; fd=os.open('/dev/tty',os.O_RDWR); print(sys.stdin.isatty() and os.isatty(fd)); os.close(fd)",
            ],
            cwd=self.main,
            env=env,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        output = b""
        try:
            while process.poll() is None:
                try:
                    output += os.read(master, 4096)
                except OSError:
                    break
            while True:
                try:
                    output += os.read(master, 4096)
                except OSError:
                    break
        finally:
            os.close(master)
            if process.poll() is None:
                process.kill()
            process.wait()

        self.assertEqual(process.returncode, 0, output)
        self.assertIn(b"True", output)

    def test_signal_reaches_executed_process(self) -> None:
        marker = self.tmp / "ready"
        code = (
            "import os,pathlib,signal,time; "
            f"pathlib.Path({str(marker)!r}).touch(); "
            "signal.signal(signal.SIGTERM, lambda *_: os._exit(42)); "
            "time.sleep(30)"
        )
        env = self._base_env()
        env["FAKE_ROOT"] = str(self.main)
        process = subprocess.Popen(
            [str(DEV), "python3", "-c", code],
            cwd=self.main,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            for _ in range(100):
                if marker.exists():
                    break
                time.sleep(0.02)
            self.assertTrue(marker.exists(), "child process did not start")
            process.send_signal(signal.SIGTERM)
            try:
                stdout, stderr = process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                self.fail(f"signal forwarding timed out; calls={self._calls()}")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()

        self.assertEqual(process.returncode, 42, (stdout, stderr))

    def test_requires_command_and_git_worktree(self) -> None:
        result = self._run(self.main)
        self.assertEqual(result.returncode, 2)
        self.assertIn("usage:", result.stderr)

        env = self._base_env()
        result = subprocess.run(
            [str(DEV), "true"],
            cwd=self.tmp,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not inside a Git worktree", result.stderr)


if __name__ == "__main__":
    unittest.main()
