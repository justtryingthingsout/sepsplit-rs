/*
    sepsplit-rs - A tool to split SEPOS firmware into its individual modules
    Copyright (C) 2024 plzdonthaxme

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#include <stdlib.h>
#ifndef SEPLIB_H
#define SEPLIB_H

/*
  Calls the main logic of the program with FFI.
  Arguments:
  - filein - the path to the extracted SEP firmware
  - outdir - the path to the output directory
  - verbose - the verbosity level (0 for no output, 1 for normal output)
  Returns:
  - 0 on success
  - 1 on failure
  Safety:
  - filein must be a null terminated char array with valid UTF-8 characters and also be a path to a file
  - outdir must be a null terminated char array with valid UTF-8 characters and also be a path to a already existing directory
*/

extern int split(const char* filein, const char* outdir, unsigned int verbose);
#endif