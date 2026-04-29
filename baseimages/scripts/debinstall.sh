#!/bin/bash
# To be run as root to prepare a bland debian base image
apt-get update
apt-get upgrade -y
apt-get install -y curl adduser bash
apt-get clean
adduser user
mkdir /project
chown -R user:user /project
chmod 755 /project
