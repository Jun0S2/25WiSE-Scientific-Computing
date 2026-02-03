#!/bin/bash

mkdir -p ex10-output    # <--- NEW

for i in ex[0-1][0-9]-[0-9][0-9][0-9][0-9][0-9][0-9]-[a-z]*-[a-z]*.zip
do
    NAME=$(basename "$i" .zip)
    bash execute.sh "$i" > "ex10-output/$NAME.out" 2>&1 &
done

wait
echo "All tests finished."
