use std::sync::Arc;

use cp_microservice::{
    api::{
        client::input_consumer::input_consumer::InputConsumer,
        shared::{request::Request, request_header::RequestHeader},
    },
    core::error::Error,
    r#impl::api::{
        client::input_consumer::amqp_input_consumer::AmqpInputConsumer, server::input::amqp_input,
        shared::amqp_queue_rpc_publisher::AmqpQueueRpcPublisher,
    },
};
use lapin::Channel;
use multiple_connections_lapin_wrapper::{
    amqp_wrapper::AmqpWrapper, config::amqp_connect_config::AmqpConnectConfig,
};
use serde_json::{json, Value};

#[tokio::main]
pub async fn main() {
    let amqp_connection_uri = std::env::args()
        .nth(1usize)
        .expect("expected amqp connection uri");

    let amqp_connection_json: String = format!("{{ \"uri\": \"{}\", \"options\": {{ \"locale\": \"en_US\", \"client_properties\": {{}} }},\"owned_tls_config\": {{}} }}", amqp_connection_uri);

    let connection_config: AmqpConnectConfig =
        serde_json::from_str(amqp_connection_json.as_str()).expect("expected connection config");
    let mut wrapper: AmqpWrapper = AmqpWrapper::try_new(connection_config)
        .expect("expected amqp wrapper from connection config");

    let channel: Arc<Channel> = wrapper
        .try_get_channel()
        .await
        .expect("expected amqp channel");

    let amqp_publisher_json: &str = r#"{
                                            "queue_name": "organization",
                                            "publish": {
                                                "exchange": "",
                                                "options": {
                                                    "mandatory": false,
                                                    "immediate": false
                                                },
                                                "properties": {
                                                    "correlation_id": "1"
                                                }
                                            },
                                            "response": {
                                                "queue": {
                                                    "name": "",
                                                    "declare": {
                                                        "options": {
                                                            "passive": false,
                                                            "durable": false,
                                                            "exclusive": false,
                                                            "auto_delete": true,
                                                            "nowait": false
                                                        },
                                                        "arguments": {}
                                                    }
                                                },
                                                "qos": {
                                                    "prefetch_count": 16,
                                                    "options": {
                                                        "global": false
                                                    }
                                                },
                                                "consume": {
                                                    "options": {
                                                        "no_local": false,
                                                        "no_ack": false,
                                                        "exclusive": false,
                                                        "nowait": false
                                                    },
                                                    "arguments": {}
                                                },
                                                "acknowledge": {
                                                    "multiple": false
                                                },
                                                "reject": {
                                                    "requeue": false
                                                }
                                            }
                                       }"#;

    let publisher: AmqpQueueRpcPublisher =
        serde_json::from_str::<AmqpQueueRpcPublisher>(amqp_publisher_json).unwrap();

    let amqp_input_consumer: AmqpInputConsumer =
        AmqpInputConsumer::new(channel, publisher, 5000u64);
    let mut request_header: RequestHeader =
        RequestHeader::new("create_org".to_string(), "1234abcd".to_string());

    let request: Request = Request::new(
        request_header,
        json!({
            "country": "es",
            "name": "example",
            "address": {
                "country": "es",
                "region": "albacete",
                "city": "villarrobledo",
                "street": "calle molino estrada",
                "number": "37",
                "additional": "",
                "postal_code": "02600"
            },
            "user_id": "simple0x47@gmail.com"
        }),
    );

    let response: Value = amqp_input_consumer.send_request(request).await.unwrap();
    println!("Response: {}", &response);
    let response_object = serde_json::from_value::<Result<String, Error>>(response).unwrap();

    let organization_id = response_object.unwrap();

    assert!(organization_id.len() > 0);
}
