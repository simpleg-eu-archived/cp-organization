#!/bin/bash

cargo build

build_code=$?

if [ $build_code -ne 0 ]; then
  echo "Build: FAILED"
  exit 1
else
  echo "Build: SUCCESS"
fi

cd ./target/debug

result_exit_code=0

# TEST CREATE ORGANIZATION SUCCESSFULLY, EXPECTED EXIT CODE: 0

DEFAULT_AMQP_CONNECTION_URI="amqp://guest:guest@127.0.0.1:5672"
TEST_AMQP_CONNECTION_URI=${TEST_AMQP_CONNECTION_URI:=$DEFAULT_AMQP_CONNECTION_URI}

./cp-organization $TEST_AMQP_CONNECTION_URI &
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
exit $result_exit_code