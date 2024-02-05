#!/usr/bin/env bash
function commit_and_push {
  message="$1"
  if [ -z "$message" ]; then
    echo "Error, message was not provided."
    exit 1
  fi
  git add .
  git commit -m "$message"
  git push
}