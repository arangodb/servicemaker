#!/bin/bash
# Now archive everything under /project
cd /project
tar czvf /tmp/output/project.tar.gz project entrypoint $1
