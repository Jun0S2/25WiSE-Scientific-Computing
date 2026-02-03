#!/bin/bash

awk '
BEGIN {
    ok = 1
    count = 0
}

# Ignore empty lines
/^[[:space:]]*$/ {
    next
}

# Valid algorithm output line
/^Algorithm[[:space:]]+[0-9]+[[:space:]]+Checksum:[[:space:]]*0x[0-9A-Fa-f]+,[[:space:]]*computed[[:space:]]*[0-9]+[[:space:]]pixel[[:space:]]in[[:space:]]*[0-9]+(\.[0-9]+)?[[:space:]]ms[[:space:]]=[[:space:]]*[0-9]+(\.[0-9]+)?[[:space:]]M[Pp]x\/s$/ {
    count++
    next
}

# Anything else is invalid
{
    ok = 0
}

END {
    # Must have at least one valid line, and no invalid lines
    if (ok && count > 0)
        exit 0
    else
        exit 1
}
'
