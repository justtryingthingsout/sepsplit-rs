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