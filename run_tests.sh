#!/bin/bash

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

# TEST CREATE ORGANIZATION SUCCESSFULLY, EXPECTED EXIT CODE: 0

DEFAULT_MONGODB_CONNECTION_URI="mongodb://127.0.0.1:27017"
export CP_ORGANIZATION_MONGODB_CONNECTION_URI=${CP_ORGANIZATION_MONGODB_CONNECTION_URI:=$DEFAULT_MONGODB_CONNECTION_URI}

DEFAULT_MONGODB_USERNAME="guest"
export CP_ORGANIZATION_MONGODB_USERNAME=${CP_ORGANIZATION_MONGODB_USERNAME:=DEFAULT_MONGODB_USERNAME}

DEFAULT_MONGODB_PASSWORD="guest"
export CP_ORGANIZATION_MONGODB_PASSWORD=${CP_ORGANIZATION_MONGODB_PASSWORD:=DEFAULT_MONGODB_PASSWORD}

DEFAULT_AMQP_CONNECTION_FILE="./config/local/amqp_connection.json"
export CP_ORGANIZATION_AMQP_CONNECTION_FILE=${CP_ORGANIZATION_AMQP_CONNECTION_FILE:=$DEFAULT_AMQP_CONNECTION_FILE}

DEFAULT_MONGODB_CONNECTION_FILE="./config/local/mongodb_connection.json"
export CP_ORGANIZATION_MONGODB_CONNECTION_FILE=${CP_ORGANIZATION_MONGODB_CONNECTION_FILE:=$DEFAULT_MONGODB_CONNECTION_FILE}

DEFAULT_AMQP_API_FILE="./config/local/amqp_api.json"
export CP_ORGANIZATION_AMQP_API_FILE=${CP_ORGANIZATION_AMQP_API_FILE:=$DEFAULT_AMQP_API_FILE}

DEFAULT_OPENID_CONNECT_CONFIG_FILE="./config/local/openid_connect_config.json"
export CP_ORGANIZATION_OPENID_CONNECT_CONFIG_FILE=${CP_ORGANIZATION_OPENID_CONNECT_CONFIG_FILE:=$DEFAULT_OPENID_CONNECT_CONFIG_FILE}

DEFAULT_AMQP_CONNECTION_URI="amqp://guest:guest@127.0.0.1:5672"
TEST_AMQP_CONNECTION_URI=${TEST_AMQP_CONNECTION_URI:=$DEFAULT_AMQP_CONNECTION_URI}

./cp-organization &
impl_pid=$!

sleep 1

# Database initialization, to be called before every integration test
db_init() {
  cd deps
  chmod +x ./db_init.sh
  ./db_init.sh
  cd ../
}
# -----------------------

db_init

./test_create_organization_successfully $TEST_AMQP_CONNECTION_URI

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