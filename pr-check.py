#!/usr/bin/env -S uv run --script
#
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "pr-check @ git+ssh://localhost/home/shahn/pr-check@main",
# ]
# ///

from pr_check import get_pr, Passes

get_pr(default_branch="main").check(Passes("cargo test"))
