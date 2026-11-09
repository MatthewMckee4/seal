# /// script
# requires-python = ">=3.11"
# dependencies = ["pyyaml"]
# ///
"""Exercise the CI gate's handling of failed, cancelled, and skipped jobs."""

import os
from pathlib import Path
import subprocess
import unittest

import yaml


class CiGateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        workflow = yaml.safe_load(
            Path(".github/workflows/ci.yml").read_text(encoding="utf-8")
        )
        cls.gate = workflow["jobs"]["ci-gate"]
        cls.step = cls.gate["steps"][0]
        cls.results = {
            key: "success" for key in cls.step["env"] if key.endswith("_RESULT")
        }

    def check_gate(self, expected_success, **overrides):
        env = {
            **os.environ,
            **self.results,
            "CODE_CHANGED": "true",
            "GITHUB_REF": "refs/pull/1/merge",
            **overrides,
        }
        result = subprocess.run(
            ["bash", "-e", "-o", "pipefail", "-c", self.step["run"]],
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode == 0, expected_success, result.stdout)

    def test_all_jobs_are_gated(self):
        workflow = yaml.safe_load(
            Path(".github/workflows/ci.yml").read_text(encoding="utf-8")
        )
        self.assertEqual(set(self.gate["needs"]), set(workflow["jobs"]) - {"ci-gate"})
        self.assertEqual(self.gate["if"], "${{ always() }}")
        self.check_gate(True)

    def test_failure_and_cancellation_block_every_job(self):
        for key in self.results:
            for result in ("failure", "cancelled", ""):
                with self.subTest(job=key, result=result):
                    self.check_gate(False, **{key: result})

    def test_skips_only_allowed_for_code_jobs_on_non_code_prs(self):
        for key in self.results:
            with self.subTest(job=key):
                self.check_gate(False, **{key: "skipped"})
                self.check_gate(
                    False,
                    GITHUB_REF="refs/heads/main",
                    CODE_CHANGED="false",
                    **{key: "skipped"},
                )
                self.check_gate(
                    key in {"CARGO_SHEAR_RESULT", "CARGO_TEST_RESULT", "COVERAGE_RESULT"},
                    CODE_CHANGED="false",
                    **{key: "skipped"},
                )

    def test_invalid_change_output_blocks(self):
        for value in ("", "unexpected"):
            with self.subTest(value=value):
                self.check_gate(False, CODE_CHANGED=value)


if __name__ == "__main__":
    unittest.main()
