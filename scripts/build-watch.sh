#!/bin/bash -u

inotifywait -e close_write,moved_to,create -m -r src lib |
while read -r directory events filename; do
    make test build
    echo GOGOGO
done
