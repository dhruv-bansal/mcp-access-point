#!/bin/bash

# Variables
ORIGINAL_REPO="https://github.com/sxhxliang/mcp-access-point.git"
BRANCH_NAME="upstream-request-body-issue"

# Script
set -e

git remote add upstream "$ORIGINAL_REPO"
git fetch upstream
git checkout -b "$BRANCH_NAME" upstream/main
git push origin "$BRANCH_NAME"