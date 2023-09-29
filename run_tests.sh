#!/bin/bash

cargo build

build_code=$?

if [ $build_code -ne 0 ]; then
  echo "Build: FAILED"
  exit 1
else
  echo "Build: SUCCESS"
fi

mv ./target/debug/* ./

result_exit_code=0

# TEST CREATE ORGANIZATION SUCCESSFULLY, EXPECTED EXIT CODE: 0

DEFAULT_AMQP_CONNECTION_FILE="./config/local/amqp_connection.json"
TEST_AMQP_CONNECTION_FILE=${TEST_AMQP_CONNECTION_FILE:=$DEFAULT_AMQP_CONNECTION_FILE}

DEFAULT_MONGODB_CONNECTION_FILE="./config/local/mongodb_connection.json"
TEST_MONGODB_CONNECTION_FILE=${TEST_MONGODB_CONNECTION_FILE:=$DEFAULT_MONGODB_CONNECTION_FILE}

DEFAULT_AMQP_API_FILE="./config/local/amqp_api.json"
TEST_AMQP_API_FILE=${TEST_AMQP_API_FILE:=$DEFAULT_AMQP_API_FILE}

DEFAULT_OPENID_CONNECT_CONFIG_FILE="./config/local/openid_connect_config.json"
TEST_OPENID_CONNECT_CONFIG_FILE=${TEST_OPENID_CONNECT_CONFIG_FILE:=$DEFAULT_OPENID_CONNECT_CONFIG_FILE}

DEFAULT_AMQP_CONNECTION_URI="amqp://guest:guest@127.0.0.1:5672"
TEST_AMQP_CONNECTION_URI=${TEST_AMQP_CONNECTION_URI:=$DEFAULT_AMQP_CONNECTION_URI}

./cp-organization $TEST_AMQP_CONNECTION_FILE $TEST_MONGODB_CONNECTION_FILE $TEST_AMQP_API_FILE $TEST_OPENID_CONNECT_CONFIG_FILE &
impl_pid=$!

sleep 1
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
rm ./cp-organization.d
rm ./test_create_organization_successfully
rm ./test_create_organization_successfully.d
rm -R ./build
rm -R ./deps
rm -R ./examples
rm -R ./incremental

exit $result_exit_code