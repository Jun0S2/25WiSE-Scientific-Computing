#!/bin/bash

mkdir -p ex07-output    # <--- NEW

for i in ex[0-1][0-9]-[0-9][0-9][0-9][0-9][0-9][0-9]-[a-z]*-[a-z]*.zip
do
    NAME=$(basename "$i" .zip)
    ./execute.sh "$i" > "ex07-output/$NAME.out" 2>&1 &
done
