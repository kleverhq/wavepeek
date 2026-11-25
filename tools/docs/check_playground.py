#!/usr/bin/env python3

"""Check native/browser waveform operations in a composed Playground/docs site."""

import argparse
import functools
import http.server
import json
import pathlib
import shlex
import subprocess
import tempfile
import threading
import urllib.parse

from playwright.sync_api import Page, sync_playwright


VCD = b"""$date today $end
$version smoke $end
$timescale 1ns $end
$scope module top $end
$var wire 1 ! clk $end
$var wire 1 " bus [1] $end
$var wire 1 # bus [0] $end
$var wire 8 $ mem[0] [7:0] $end
$upscope $end
$enddefinitions $end
#0
0!
0"
0#
b11110000 $
#5
1!
0"
1#
b01010101 $
#10
0!
0"
0#
b00001111 $
"""


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *_args: object) -> None:
        pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--site", required=True, type=pathlib.Path)
    parser.add_argument("--native-bin", required=True, type=pathlib.Path)
    return parser.parse_args()


def run_native(binary: pathlib.Path, cwd: pathlib.Path, command: str) -> tuple[int, str, str]:
    result = subprocess.run(
        [str(binary), *shlex.split(command)],
        cwd=cwd,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode, result.stdout, result.stderr


def run_browser(page: Page, command: str) -> tuple[int, str, str]:
    entries = page.locator("#transcript .playground__entry")
    before = entries.count()
    page.locator("#command-line").fill(command)
    page.locator("#run").click()
    assert entries.count() == before + 1
    entry = entries.first
    entry.locator(".playground__duration").filter(has_not_text="Running").wait_for(
        timeout=30_000
    )
    return (
        int(entry.get_attribute("data-exit-status")),
        entry.locator(".playground__stdout").text_content(),
        entry.locator(".playground__stderr").text_content(),
    )


def check_reads(
    page: Page, native_bin: pathlib.Path, cwd: pathlib.Path, filename: str, signal: str,
    first: str, later: str, end: str,
) -> None:
    commands = [
        f"value --waves {filename} --signals {signal} --at {later}",
        f"value --waves {filename} --signals {signal},{signal} --at {later},{first},{later} --json",
        f"property --waves {filename} --on 'posedge {signal}' --eval '{signal} == 1' "
        f"--sample-mode native --to {end} --capture match --json",
        f"change --waves {filename} --signals {signal} --on '*' "
        f"--sample-mode native --to {end} --max 1 --jsonl",
    ]
    for index, command in enumerate(commands):
        actual = run_browser(page, command)
        assert actual == run_native(native_bin, cwd, command), command
        status, stdout, stderr = actual
        expected_stderr = "info: opening waveform\ninfo: sampling values\n" if index == 0 else ""
        assert status == 0 and stdout.strip() and stderr == expected_stderr, (command, actual)
        if index in (1, 2):
            result = json.loads(stdout)
            assert result["type"] == "result" and result["data"], command
            if index == 1:
                assert len(result["data"]) == 3, command
                assert result["data"][0] == result["data"][2], command
        elif index == 3:
            records = [json.loads(line) for line in stdout.splitlines()]
            assert records[0]["type"] == "begin" and records[-1]["type"] == "end"
            assert sum(record["type"] == "data" for record in records) == 1
            assert records[-1]["summary"] == {
                "complete": False, "returned": 1, "limit": 1, "total": None,
            }

    command = f"value --waves {filename} --signals missing --at {later} --json"
    actual = run_browser(page, command)
    assert actual == run_native(native_bin, cwd, command), command
    status, stdout, stderr = actual
    fatal = json.loads(stdout)
    assert status == 1 and not stderr
    assert fatal["type"] == "fatal" and fatal["code"] == "WPK-F0004"


def check(site: pathlib.Path, native_bin: pathlib.Path) -> None:
    assert site.is_dir(), f"site does not exist: {site}"
    assert native_bin.is_file(), f"native binary does not exist: {native_bin}"

    server = http.server.ThreadingHTTPServer(
        ("127.0.0.1", 0), functools.partial(QuietHandler, directory=site)
    )
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    root = f"{origin}/wavepeek/"
    demo_dir = site / "wavepeek" / "assets" / "playground"

    try:
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch()
            page = browser.new_page()
            # Keep the local smoke test independent of remote assets such as fonts.
            page.route(lambda url: not url.startswith(f"{origin}/"), lambda route: route.abort())
            page.goto(root, wait_until="networkidle")
            page.locator("#source-status").filter(has_text="Ready").wait_for()

            version = subprocess.run(
                [str(native_bin), "--version"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.split()[-1].removeprefix("v")
            assert page.locator(".playground").get_attribute("data-version") == version

            info = "info --waves scr1_axi.fst"
            assert run_browser(page, info) == run_native(native_bin, demo_dir, info)
            check_reads(
                page, native_bin, demo_dir, "scr1_axi.fst", "TOP.clk", "1ps", "2ps", "200ps"
            )

            page.locator("a", has_text="Documentation").first.click()
            page.wait_for_url(f"{origin}/wavepeek/latest/**")
            assert page.locator("main").text_content().strip()
            assert page.locator(".playground").count() == 0

            page.goto(root, wait_until="networkidle")
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            requests: list[str] = []
            page.on("request", lambda request: requests.append(request.url))
            page.locator("#local-file").set_input_files(
                {"name": "local.vcd", "mimeType": "text/plain", "buffer": VCD}
            )
            page.locator("#source-status").filter(has_text="Ready").wait_for()
            status, stdout, stderr = run_browser(page, "info --waves local.vcd")
            assert status == 0 and stderr == "info: opening waveform\n" and "time_unit: 1ns" in stdout
            with tempfile.TemporaryDirectory() as directory:
                cwd = pathlib.Path(directory)
                (cwd / "local.vcd").write_bytes(VCD)
                check_reads(
                    page, native_bin, cwd, "local.vcd", "top.clk", "0ns", "5ns", "10ns"
                )
                check_reads(
                    page, native_bin, cwd, "local.vcd", "top.bus", "0ns", "5ns", "10ns"
                )
                command = (
                    "value --waves local.vcd --scope top "
                    "--signals 'mem[0][7:4],bus[1:0],mem[0][7:4]' "
                    "--at 0ns,5ns,0ns --json"
                )
                actual = run_browser(page, command)
                assert actual == run_native(native_bin, cwd, command), command
                status, stdout, stderr = actual
                assert status == 0 and not stderr
                rows = json.loads(stdout)["data"]
                assert len(rows) == 3 and rows[0] == rows[2]
                assert rows[1]["signals"][0]["path"] == "top.mem[0][7:4]"
                assert rows[1]["signals"][0]["value"] == "4'h5"

            assert not [
                url
                for url in requests
                if urllib.parse.urlparse(url).netloc
                != urllib.parse.urlparse(origin).netloc
            ]
            browser.close()
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def main() -> int:
    args = parse_args()
    check(args.site.resolve(), args.native_bin.resolve())
    print("Playground browser smoke passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
