

use zeromq::ZmqMessage;


pub fn zmq_message_pretty_print(msg:ZmqMessage) -> String{
    let mut result = String::new();
    result.push_str("ZmqMessage: [");
    for item in msg.iter() {
        result.push_str(&format!("\n\t{:?}", item));
    }
    result.push_str("\n]");
    result
}