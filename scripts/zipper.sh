#!/bin/bash

# This program finds the files which have been modified or added to the
# base image in the virtual environment in /home/python/the_venv and then
# it adds the files in the directory /home/python/project . Finally, it
# zips all these files and writes the result to /tmp/output/project.tar.gz .

# First find all files which have changed:
sha256sum -c sums_sha256 2> /dev/null | grep -v OK | awk '{ split($1, x, ":"); print(x[1]) }' > /tmp/changedfiles

# Now find all files which have been added:
cd /home/python
find the_venv -type f | xargs sha256sum >> sums_sha256_new
cat sums_sha256_new sums_sha256 | sort | uniq -c | grep "^ *1" | awk '{ print $3 }' > /tmp/newfiles

# Now archive everything plus /home/python/project:
tar czvf /tmp/output/project.tar.gz project --verbatim-files-from --files-from /tmp/changedfiles --files-from /tmp/newfiles
