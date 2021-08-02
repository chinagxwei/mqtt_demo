use crate::types::TypeKind;
use crate::tools::un_pack_tool::{get_type, get_protocol_name_and_version};
use crate::message::v3::{
    ConnackMessage, ConnectMessage, DisconnectMessage, MqttMessageV3,
    PubackMessage, PubcompMessage, PublishMessage, PubrecMessage, PubrelMessage,
    SubackMessage, SubscribeMessage, UnsubackMessage, UnsubscribeMessage,
};
use crate::protocol::{MqttProtocolLevel, MqttDup, MqttQos, MqttRetain};
use crate::hex::PropertyItem;
use crate::tools::pack_tool::pack_header;
use crate::packet::v3_unpacket;

pub mod v3;
pub mod v5;

#[derive(Debug)]
pub enum MqttMessageKind {
    Response(Vec<u8>),
    Responses(Vec<Vec<u8>>),
    RequestV3(MqttMessageV3),
    RequestsV3(Vec<MqttMessageV3>),
    RequestV5,
    Exit(Vec<u8>),
}

impl MqttMessageKind {
    pub fn is_v3(&self) -> bool {
        matches!(self, MqttMessageKind::RequestV3(_))
    }

    pub fn is_v3s(&self) -> bool {
        matches!(self, MqttMessageKind::RequestsV3(_))
    }

    pub fn get_v3(&self) -> Option<&MqttMessageV3> {
        match self {
            MqttMessageKind::RequestV3(kind) => {
                Some(kind)
            }
            _ => { None }
        }
    }

    pub fn get_v3s(&self) -> Option<&Vec<MqttMessageV3>> {
        match self {
            MqttMessageKind::RequestsV3(kind) => {
                Some(kind)
            }
            _ => { None }
        }
    }
}

impl MqttMessageKind {
    pub fn v3(base_msg: BaseMessage) -> Option<MqttMessageKind> {
        match base_msg.get_message_type() {
            TypeKind::CONNECT => { Some(Self::RequestV3(MqttMessageV3::Connect(ConnectMessage::from(base_msg)))) }
            TypeKind::CONNACK => { Some(Self::RequestV3(MqttMessageV3::Connack(ConnackMessage::from(base_msg)))) }
            TypeKind::PUBLISH => { Some(Self::RequestV3(MqttMessageV3::Publish(PublishMessage::from(base_msg)))) }
            TypeKind::PUBACK => { Some(Self::RequestV3(MqttMessageV3::Puback(PubackMessage::from(base_msg)))) }
            TypeKind::PUBREC => { Some(Self::RequestV3(MqttMessageV3::Pubrec(PubrecMessage::from(base_msg)))) }
            TypeKind::PUBREL => { Some(Self::RequestV3(MqttMessageV3::Pubrel(PubrelMessage::from(base_msg)))) }
            TypeKind::PUBCOMP => { Some(Self::RequestV3(MqttMessageV3::Pubcomp(PubcompMessage::from(base_msg)))) }
            TypeKind::SUBSCRIBE => {
                let mut subs = v3_unpacket::subscribe(base_msg);
                let res = subs.into_iter()
                    .map(|x| MqttMessageV3::Subscribe(x))
                    .collect::<Vec<MqttMessageV3>>();
                Some(Self::RequestsV3(res))
            }
            // TypeKind::SUBACK => { Some(Self::RequestV3(MqttMessageV3::Suback(SubackMessage::from(base_msg)))) }
            TypeKind::UNSUBSCRIBE => { Some(Self::RequestV3(MqttMessageV3::Unsubscribe(UnsubscribeMessage::from(base_msg)))) }
            TypeKind::UNSUBACK => { Some(Self::RequestV3(MqttMessageV3::Unsuback(UnsubackMessage::from(base_msg)))) }
            TypeKind::PINGREQ => { Some(Self::RequestV3(MqttMessageV3::Pingresp(PingrespMessage::default()))) }
            TypeKind::DISCONNECT => { Some(Self::RequestV3(MqttMessageV3::Disconnect((DisconnectMessage::default())))) }
            TypeKind::AUTH => { None }
            _ => { None }
        }
    }
}


pub trait MqttMessage {
    fn get_message_type(&self) -> TypeKind;
}

pub trait MqttBytesMessage: MqttMessage {
    fn as_bytes(&self) -> &[u8];
}

#[derive(Debug)]
pub struct BaseMessage {
    pub msg_type: TypeKind,
    pub dup: Option<MqttDup>,
    pub qos: Option<MqttQos>,
    pub retain: Option<MqttRetain>,
    pub bytes: Vec<u8>,
}

impl MqttMessage for BaseMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<Vec<u8>> for BaseMessage {
    fn from(data: Vec<u8>) -> Self {
        let (mut r#type2, retain, qos, dup, _last_bytes) = get_type(data.as_slice());
        BaseMessage { msg_type: r#type2.unwrap(), dup, qos, retain, bytes: data }
    }
}

impl From<&[u8]> for BaseMessage {
    fn from(data: &[u8]) -> Self {
        let (mut r#type2, retain, qos, dup, _last_bytes) = get_type(data);
        BaseMessage { msg_type: r#type2.unwrap(), dup, qos, retain, bytes: data.to_vec() }
    }
}

#[derive(Debug)]
pub struct BaseConnect {
    msg_type: TypeKind,
    protocol_name: String,
    protocol_level: MqttProtocolLevel,
}

impl BaseConnect {
    pub fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }

    pub fn get_protocol_level(&self) -> MqttProtocolLevel {
        self.protocol_level
    }

    pub fn get_protocol_name(&self) -> String {
        self.protocol_name.to_owned()
    }
}

impl From<&BaseMessage> for BaseConnect {
    fn from(data: &BaseMessage) -> Self {
        let message_bytes = data.bytes.get(2..).unwrap();
        let (
            mut protocol_name,
            mut protocol_level
        ) = get_protocol_name_and_version(message_bytes);
        BaseConnect {
            msg_type: data.msg_type,
            protocol_name: protocol_name.unwrap(),
            protocol_level: protocol_level.unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectMessagePayload {
    pub client_id: String,
    pub will_topic: Option<String>,
    pub will_message: Option<String>,
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub properties: Option<Vec<PropertyItem>>,
}


#[derive(Debug, Clone)]
pub struct PingreqMessage {
    msg_type: TypeKind,
    bytes: Vec<u8>,
}

impl MqttMessage for PingreqMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<BaseMessage> for PingreqMessage {
    fn from(mut base: BaseMessage) -> Self {
        PingreqMessage { msg_type: base.msg_type, bytes: base.bytes }
    }
}

impl MqttBytesMessage for PingreqMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()
    }
}

impl Default for PingreqMessage {
    fn default() -> Self {
        PingreqMessage {
            msg_type: TypeKind::PINGREQ,
            bytes: pack_header(TypeKind::PINGREQ, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PingrespMessage {
    msg_type: TypeKind,
    bytes: Vec<u8>,
}

impl MqttMessage for PingrespMessage {
    fn get_message_type(&self) -> TypeKind {
        self.msg_type
    }
}

impl From<BaseMessage> for PingrespMessage {
    fn from(mut base: BaseMessage) -> Self {
        PingrespMessage { msg_type: base.msg_type, bytes: base.bytes }
    }
}

impl MqttBytesMessage for PingrespMessage {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()
    }
}

impl Default for PingrespMessage {
    fn default() -> Self {
        PingrespMessage {
            msg_type: TypeKind::PINGRESP,
            bytes: pack_header(TypeKind::PINGRESP, 0),
        }
    }
}
