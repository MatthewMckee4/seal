# seal

A fast, simple Python release management tool written in Rust.

## What is seal?

seal unifies the Python release cycle into a single command.

## The Problem

Python release management is fragmented. You need bump2version for versioning, towncrier for changelogs, manual git tagging, twine for publishing - each with their own configuration and workflow. seal aims to solve this by providing one tool that handles the entire release cycle.

## Status

Early development - not yet functional

## Vision

seal aims to make Python releases as simple as running a single command, with sensible defaults that work for most projects and clear configuration for those that need it.

---

Built with Rust. Inspired by uv, ruff, and the need for simpler Python releases.
