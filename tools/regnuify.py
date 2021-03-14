import sys

OLD = '''// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.'''

NEW = '''// syn.txt -- a text based synthesizer and audio workstation
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
// along with this program.  If not, see <https://www.gnu.org/licenses/>.'''

if __name__ == '__main__':
    path = sys.argv[1]
    print(f"Regnuifying {path}")
    
    with open(path, mode='r+', encoding='utf-8') as f:
        src = f.read()
        dst = src.replace(OLD, NEW)
        f.truncate()
        f.seek(0)
        f.write(dst)
