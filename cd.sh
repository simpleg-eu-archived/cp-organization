#!/bin/bash

CP_ORGANIZATION_SSH_PRIVATE_KEY=$(bws secret get "$CP_ORGANIZATION_SSH_KEY_SECRET" --access-token "$CP_ORGANIZATION_SECRETS_MANAGER_ACCESS_TOKEN" | jq -r '.value')

mkdir ~/.ssh
echo "$CP_ORGANIZATION_SSH_PRIVATE_KEY" > ~/.ssh/cp_organization_ssh_private_key
chmod 600 ~/.ssh/cp_organization_ssh_private_key

eval `ssh-agent`
ssh-add ~/.ssh/*

ssh -o StrictHostKeyChecking=accept-new $CP_ORGANIZATION_SSH_USERNAME@$CP_ORGANIZATION_SSH_HOSTNAME -p 68  < cd_remote.sh