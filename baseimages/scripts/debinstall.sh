#!/bin/bash
# To be run as root to prepare base image
apt-get update
apt-get upgrade -y

apt-get install -y --no-install-recommends \
    curl \
    adduser \
    bash \
    python3 \
    python3-venv \
    python3-pip

apt-get clean
rm -rf /var/lib/apt/lists/*

adduser user
mkdir /project
chown -R user:user /project
chmod 755 /project
