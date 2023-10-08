FROM debian:12
COPY ./target/release/cp-organization .
RUN apt-get update && apt-get install -y libssl-dev ca-certificates
ENTRYPOINT [ "./cp-organization" ]