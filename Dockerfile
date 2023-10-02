FROM debian:12
COPY ./target/release/cp-organization .
ENTRYPOINT [ "cp-organization", "$AMQP_CONNECTION_FILE", "$MONGODB_CONNECTION_FILE", "$AMQP_API_FILE" ]