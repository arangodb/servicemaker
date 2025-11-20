#!/bin/bash
# Now archive everything under /project
cd /project
tar czvf /tmp/output/output/project.tar.gz the_venv entrypoint $1
