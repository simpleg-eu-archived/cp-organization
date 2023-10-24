#!/bin/bash

cd ~/cuplan/cp-organization

git pull
sudo docker compose pull
sudo docker compose stop cp-organization
sudo docker compose rm -f cp-organization
sudo -E docker compose up -d