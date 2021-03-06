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
// Copyright (C) 2021  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

EOF
    cat "$filename" >> "$temp"
    cp "$temp" "$filename"
    rm "$temp"
}

for f in $(find . -name '*.rs'); do
    gunify "$f"
done