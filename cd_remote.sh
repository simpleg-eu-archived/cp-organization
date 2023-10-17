#!/bin/bash

cd ~/cuplan/cp-organization/self

git pull
sudo docker compose pull
sudo docker compose stop
sudo docker compose rm -f
sudo docker compose up -d