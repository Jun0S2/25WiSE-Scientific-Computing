#!/bin/bash

INST="$1"

case "$INST" in
    a01_cholesky|bcspwr04|hor__131|impcol_a|mcca|young1c)
        # Accept general format:
        # name: err_max=<float>, err_2=<float>
        grep -Eq "^${INST}:[[:space:]]*err_max=[[:space:]]*[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?,[[:space:]]*err_2=[[:space:]]*[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?$"
        ;;
    *)
        echo "Unknown instance: $INST"
        exit 1
        ;;
esac
