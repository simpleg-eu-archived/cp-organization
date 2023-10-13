FROM debian:12
COPY ./target/release/cp-organization .
COPY ./config ./config
RUN apt-get update && apt-get install -y libssl-dev ca-certificates curl unzip
# Bitwarden secrets manager
RUN curl -LO https://github.com/bitwarden/sdk/releases/download/bws-v0.3.0/bws-x86_64-unknown-linux-gnu-0.3.0.zip && unzip bws-x86_64-unknown-linux-gnu-0.3.0.zip && mv bws /usr/local/bin
ENTRYPOINT [ "./cp-organization", "./config/prod/amqp_api.json", "./config/prod/openid_connect_config.json" ]