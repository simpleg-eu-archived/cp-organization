FROM debian:12
COPY ./target/release/cp-organization .
RUN apt-get update && apt-get install -y libssl-dev
ENTRYPOINT [ "./cp-organization", "$AMQP_CONNECTION_FILE", "$MONGODB_CONNECTION_FILE", "$AMQP_API_FILE", "$OPENID_CONNECT_CONFIG" ]