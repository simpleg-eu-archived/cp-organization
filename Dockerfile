FROM debian:12
COPY ./target/release/cp-organization .
COPY ./config ./config
RUN apt-get update && apt-get install -y libssl-dev ca-certificates
ENTRYPOINT [ "./cp-organization", "./config/prod/amqp_api.json", "./config/prod/openid_connect_config.json" ]