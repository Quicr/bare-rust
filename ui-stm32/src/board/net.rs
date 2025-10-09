use heapless::String;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ui_app::{Frame, FromNet, ToNet, FRAME_SIZE, MAX_MESSAGE_LEN};

// XXX(RLB) This reader will only work as long as the data in the value of the TLV doesn't have any
// characters that need to be escaped according to SLIP.  Also, it only supports TLVs with up to
// 128 bytes of payload.
pub struct NetRx<'a, Reader> {
    rx: &'a mut Reader,
}

impl<'a, Reader> NetRx<'a, Reader>
where
    Reader: embedded_io_async::Read + 'a,
{
    pub fn new(rx: &'a mut Reader) -> Self {
        Self { rx }
    }

    pub async fn next(&mut self) -> Option<FromNet> {
        Some(FromNet::read_tlv(&mut self.rx).await)
    }
}

pub struct NetTx<Writer> {
    tx: Writer,
}

impl<Writer> NetTx<Writer> {
    pub fn new(tx: Writer) -> Self {
        Self { tx }
    }
}

impl<Writer> ui_app::NetTx for NetTx<Writer>
where
    Writer: embedded_io::Write,
{
    fn write(&mut self, to_net: &ToNet) {
        to_net.write_tlv(&mut self.tx);
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
enum PacketTypeToNet {
    Message = 0x09,
    Ping = 0x0e,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
enum PacketTypeFromNet {
    Message = 0x09,
    Pong = 0x0f,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
enum ChannelId {
    Ptt = 0,
    PttAi = 1,
    Chat = 2,
    ChatAi = 3,
    Count = 4,
}

trait TlvRead {
    async fn read_tlv(r: &mut impl embedded_io_async::Read) -> Self;
}

impl TlvRead for FromNet {
    async fn read_tlv(r: &mut impl embedded_io_async::Read) -> Self {
        let mut type_length = [0u8; 5];
        r.read_exact(&mut type_length).await.unwrap();

        let packet_type = PacketTypeFromNet::try_from(type_length[0]).unwrap();
        let len = u32::from_be_bytes(type_length[1..].try_into().unwrap()) as usize;

        match packet_type {
            PacketTypeFromNet::Pong => Self::Pong,
            PacketTypeFromNet::Message => {
                let mut channel_id = [0];
                r.read(&mut channel_id).await.unwrap();
                let channel_id = ChannelId::try_from(channel_id[0]).unwrap();

                match channel_id {
                    ChannelId::Ptt => {
                        let mut frame_data = [0u8; 2 * FRAME_SIZE];
                        assert_eq!(len, frame_data.len() + 1);
                        r.read_exact(&mut frame_data).await.unwrap();

                        let mut frame = Frame::default();
                        for (x8, x16) in frame_data.chunks(2).zip(frame.0.iter_mut()) {
                            *x16 = u16::from_be_bytes(x8.try_into().unwrap());
                        }

                        Self::AudioFrame(frame)
                    }
                    ChannelId::Chat => {
                        let mut msg = [0; MAX_MESSAGE_LEN];
                        assert!(len < msg.len());

                        r.read_exact(&mut msg[..len]).await.unwrap();
                        Self::Chat(String::from_iter(msg.into_iter().map(|b| char::from(b))))
                    }

                    _ => unreachable!("Unsupported Channel ID"),
                }
            }
        }
    }
}

trait TlvWrite {
    fn write_tlv(&self, w: &mut impl embedded_io::Write);
}

impl TlvWrite for ToNet {
    fn write_tlv(&self, w: &mut impl embedded_io::Write) {
        todo!();
    }
}

/*
trait Tlv {
    const TYPE: PacketType;
    fn len(&self) -> usize;
    fn write_payload(&self, w: &mut impl embedded_io::Write);
    fn write(&self, w: &mut impl embedded_io::Write) {
        let len = self.len() as u32;
        w.write(&[Self::TYPE.into()]).unwrap();
        w.write(&len.to_be_bytes()).unwrap();
        self.write_payload(w);
    }
}

struct Ping;

impl Tlv for Ping {
    const TYPE: PacketType = PacketType::Ping;

    fn len(&self) -> usize {
        0
    }

    fn write_payload(&self, _w: &mut impl embedded_io::Write) {}
}

trait MessageBody {
    const MESSAGE_TYPE: MessageType;
    fn len(&self) -> usize;
    fn write(&self, w: &mut impl embedded_io::Write);
}

struct MediaMessage<'a> {
    is_last: bool,
    payload: &'a [u8],
}

impl<'a> MessageBody for MediaMessage<'a> {
    const MESSAGE_TYPE: MessageType = MessageType::Media;

    fn len(&self) -> usize {
        // is_last + payload_len + payload
        1 + 4 + self.payload.len()
    }

    fn write(&self, w: &mut impl embedded_io::Write) {
        let len = self.payload.len() as u32;
        w.write(&[self.is_last.into()]).unwrap();
        w.write(&len.to_be_bytes()).unwrap();
        w.write(self.payload).unwrap();
    }
}

struct ChatMessage<'a> {
    payload: &'a [u8],
}

impl<'a> MessageBody for ChatMessage<'a> {
    const MESSAGE_TYPE: MessageType = MessageType::Chat;

    fn len(&self) -> usize {
        // payload_len + payload
        4 + self.payload.len()
    }

    fn write(&self, w: &mut impl embedded_io::Write) {
        let len = self.payload.len() as u32;
        w.write(&len.to_be_bytes()).unwrap();
        w.write(self.payload).unwrap();
    }
}

struct Message<B: MessageBody> {
    channel_id: ChannelId,
    body: B,
}

impl<B: MessageBody> Tlv for Message<B> {
    const TYPE: PacketType = PacketType::Message;

    fn len(&self) -> usize {
        // ChannelId + MessageType + Body
        2 + self.body.len()
    }

    fn write_payload(&self, w: &mut impl embedded_io::Write) {
        w.write(&[self.channel_id.into(), B::MESSAGE_TYPE.into()])
            .unwrap();
        self.body.write(w);
    }
}
*/
