#!/bin/bash
echo "==================================================================="
ZIP="$1"
NAME=$(basename "$ZIP" .zip)

SCRIPT_DIR=$(pwd)
CORRECT="$SCRIPT_DIR/correct.sh"

# Unzip student code
unzip -u "$ZIP" "$NAME/Cargo.toml" "$NAME/src/*" >/dev/null

echo "$NAME :"
cd "$NAME" || { echo "Cannot cd into $NAME"; exit 1; }

(
    ulimit -t 7200

    echo "Compiling..."
    if ! cargo build --release >/dev/null 2>&1; then
        echo "Build FAILED"
        exit 1
    fi

    echo "Running test..."

    OUTFILE="${NAME}.out"
    PGM="${NAME}.pgm"
    BIN="$(pwd)/target/release/${NAME}"

    echo "--------------------------------------------------------"

    # --------------------------------------------------------
    # Run compiled binary with MPI (safe)
    # --------------------------------------------------------
    mpirun -np 4 "$BIN" "$PGM" 2> "$OUTFILE"


    echo
    echo "Student output:"
    cat "$OUTFILE"
    echo

    echo "Expected format:"
    echo "Algorithm 1 Checksum: <hex>, computed <integer> pixel in <float> ms = <float> Mpx/s"
    echo "Algorithm 2 Checksum: <hex>, computed <integer> pixel in <float> ms = <float> Mpx/s"
    echo " ..."

    # --------------------------------------------------------
    # Correctness check
    # --------------------------------------------------------
    bash "$CORRECT" < "$OUTFILE"
    if [ $? -eq 0 ]; then
        echo "*** OUTPUT FORMAT OK ***"
    else
        echo "== OUTPUT FORMAT FAIL =="
    fi

    echo
    echo "Parallelization check:"

    # --------------------------------------------------------
    # Rayon detection
    # --------------------------------------------------------
    if grep -Eq '^[[:space:]]*rayon[[:space:]]*=' Cargo.toml; then
        if grep -R "rayon" -q src/; then
            echo "  Rayon: OK"
        else
            echo "  Rayon: declared but NOT used"
        fi
    else
        echo "  Rayon: NOT found"
    fi

    # --------------------------------------------------------
    # MPI detection
    # --------------------------------------------------------
    if grep -Eq '^[[:space:]]*mpi[[:space:]]*=' Cargo.toml; then
        if grep -R "mpi" -q src/; then
            echo "  MPI: OK"
        else
            echo "  MPI: declared but NOT used"
        fi
    else
        echo "  MPI: NOT found"
    fi

    echo
    rm -f "$OUTFILE"
)

cargo clippy
cd "$SCRIPT_DIR"

