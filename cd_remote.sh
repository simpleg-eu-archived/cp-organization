#!/bin/bash

cd ~/cuplan/cp-organization/self

git pull
sudo docker compose pull
sudo docker compose stop
sudo docker compose rm -f -v
sudo -E docker compose up -d