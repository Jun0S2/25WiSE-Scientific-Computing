#!/bin/bash

# Check if exactly one argument (filename) is provided
# $# : number of arguments passed to the script
# -ne : "not equal"
# $0 : script name
if [ $# -ne 1 ]; then
    # Print usage instruction if wrong number of arguments
    echo "Usage: $0 <Data_SC.gz>"
    # Exit with error code 1
    exit 1
fi

# Save the first argument (filename) into a variable
# No spaces around '=' in Bash variable assignment
file=$1

# Compute minimum value
# gzip works for both linux and macOS
# tr -d '\r' < "$file" : remove Windows CRLF line endings
# cut -d'|' -f2 : extract the second column using '|' delimiter
# sort -n : sort numerically
# head -1 : get the first line (minimum)
# $(...) : command substitution, store result in variable
min=$(gzip -cd "$file" | tr -d '\r' | cut -d'|' -f2 | sort -n | head -1)

# Compute maximum value
# tail -1 : get the last line (maximum) after numeric sort
max=$(gzip -cd "$file" | tr -d '\r' | cut -d'|' -f2 | sort -n | tail -1)

# Print results
echo "$min"
echo "$max"

