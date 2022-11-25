#!/bin/bash
set -e

commit_msg=$(git log -1 --pretty=%B)

if [[ "$commit_msg" == "Release"* ]]; then
  echo 1
else
  echo 0
fi