#!/usr/bin/env bash

# gnuify: easily prepend a GNU license header to each source file.

gunify() {
    filename=$1
    if grep -q 'This program is free software' "$filename"; then
        echo "skipping $filename"
        return
    fi
    echo "gunifying $filename"
    temp=$(mktemp)
    cat > "$temp" <<EOF
// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

EOF
    cat "$filename" >> "$temp"
    cp "$temp" "$filename"
    rm "$temp"
}

for f in $(find . -name '*.rs'); do
    gunify "$f"
done