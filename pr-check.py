#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "pr-check @ git+ssh://localhost/home/shahn/pr-check@main",
# ]
# ///

import json
import subprocess
from dataclasses import dataclass
from pathlib import Path

from pr_check import get_pr, Rule, Ok, Err

type Result[T, E] = Ok[T] | Err[E]


@dataclass
class MinCoverage(Rule):
    min_percent: float

    def check(self, cwd: Path) -> Result[None, str]:
        result = subprocess.run(
            [
                "nix", "develop", f"{cwd}",
                "--extra-experimental-features", "nix-command flakes",
                "-c", "cargo", "llvm-cov", "--json",
                "--ignore-filename-regex=rustc|\\.cargo",
            ],
            capture_output=True,
            cwd=cwd,
        )
        if result.returncode != 0:
            return Err(f"coverage command failed:\n{result.stderr.decode()}")
        data = json.loads(result.stdout)
        percent = data["data"][0]["totals"]["lines"]["percent"]
        if percent < self.min_percent:
            return Err(
                f"Line coverage {percent:.1f}% is below minimum {self.min_percent:.1f}%"
            )
        return Ok(None)


get_pr(default_branch="main").check(MinCoverage(min_percent=80))
