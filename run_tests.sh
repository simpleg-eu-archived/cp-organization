#!/bin/bash
echo "cp-organization: tests runner"

if [[ -z "$CP_ORGANIZATION_SECRETS_MANAGER_ACCESS_TOKEN" ]]; then
  echo "CP_ORGANIZATION_SECRETS_MANAGER_ACCESS_TOKEN is not set."
  exit 1
fi

if [[ $CP_ENVIRONMENT -eq 0 ]]; then
  echo "Local development mode"
  export $(cat ./env/local.env | xargs)
elif [[ $CP_ENVIRONMENT -eq 1 ]]; then
  echo "Github Actions development mode"
  export $(cat ./env/actions.env | xargs)
else
  echo "Default local development mode"
  export $(cat ./env/local.env | xargs)
fi

cargo build

build_code=$?

if [ $build_code -ne 0 ]; then
  echo "Build: FAILED"
  exit 1
else
  echo "Build: SUCCESS"
fi

mv ./target/debug/cp-organization ./cp-organization
mv ./target/debug/test_create_organization_successfully ./test_create_organization_successfully

result_exit_code=0

DEFAULT_AMQP_API_FILE="./config/dev/amqp_api.json"
export CP_ORGANIZATION_AMQP_API_FILE=${CP_ORGANIZATION_AMQP_API_FILE:=$DEFAULT_AMQP_API_FILE}

./cp-organization $CP_ORGANIZATION_AMQP_API_FILE &
impl_pid=$!

sleep 1

# Database initialization, to be called before every integration test
db_init() {
  cd compose
  chmod +x ./db_init.sh
  source db_init.sh
  cd ../
}
# -----------------------

db_init

# TEST CREATE ORGANIZATION SUCCESSFULLY, EXPECTED EXIT CODE: 0

CP_ORGANIZATION_TEST_AMQP_CONNECTION_URI=$(bws secret get $CP_ORGANIZATION_TEST_AMQP_CONNECTION_URI_SECRET --access-token $CP_ORGANIZATION_SECRETS_MANAGER_ACCESS_TOKEN | jq -r '.value')

./test_create_organization_successfully $CP_ORGANIZATION_TEST_AMQP_CONNECTION_URI

test_create_organization_successfully_code=$?
if [ $test_create_organization_successfully_code -eq 0 ]; then
  echo "Test create organization successfully: SUCCESS"
else
  echo "Test create organization successfully: FAILED"
  result_exit_code=1
fi

kill $impl_pid

rm ./cp-organization
rm ./test_create_organization_successfully

exit $result_exit_code