#!/bin/bash
# Now archive everything under /project
cd /project
tar czvf /tmp/project.tar.gz the_venv entrypoint $1
