#!/bin/bash
echo "==================================================================="
ZIP="$1"
NAME=$(basename "$ZIP" .zip)

SCRIPT_DIR=$(pwd)
MATRIX_DIR="$SCRIPT_DIR/matrix_instances"
CORRECT="$SCRIPT_DIR/correct.sh"

# Unzip student code
unzip -u "$ZIP" "$NAME/Cargo.toml" "$NAME/src/*"

echo "$NAME :"
cd "$NAME" || { echo "Cannot cd into $NAME"; exit 1; }

(
    ulimit -t 7200

    echo "Compiling..."
    if ! cargo build --release >/dev/null 2>&1; then
        echo "Build FAILED"
        exit 1
    fi

    echo "Running tests on all .mtx matrices..."
    for g in "$MATRIX_DIR"/*.mtx
    do
        INST=$(basename "$g" .mtx)
        OUTFILE="${NAME}_${INST}.out"

        echo "--------------------------------------------------------"
        echo "Testing $INST"

        # Run: cargo run <matrix> 2
        time cargo run --release "$g" 2 > "$OUTFILE" 2>&1

        echo
        echo "Student output:"
        cat "$OUTFILE"
        echo

        echo "Expected format:"
        echo "${INST}: err_max=<float>, err_2=<float>"
        echo

        # Correctness check
        "$CORRECT" "$INST" < "$OUTFILE"
        if [ $? -eq 0 ]; then
            echo "*** OK ***"
        else
            echo "== FAIL =="
        fi

        echo
        rm -f "$OUTFILE"
    done
)

cargo clippy
cd "$SCRIPT_DIR"
