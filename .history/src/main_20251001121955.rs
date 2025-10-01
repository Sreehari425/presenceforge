use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use serde_json::json;
use serde_json::Value;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::process;
use anyhow::{Result, anyhow};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Client<'a> {
    client_id: &'a str,
    #[cfg(unix)]
    socket_port: UnixStream,
}

trait DiscordIPC {
    fn send(&mut self, opcode: u32, payload: &Value) -> Result<()>;
    fn recv(&mut self) -> Result<(u32, Value)>;
    fn handshake(&mut self) -> Result<()>;
    fn set_activity(&mut self, activity: Value) -> Result<()>;
    fn unset_activity(&mut self) -> Result<()>;
    fn close(&mut self);
}

impl<'a> Client<'a> {
    fn new(client_id: &'a str, socket_port: UnixStream) -> Self {
        Self {
            client_id,
            socket_port,
        }
    }
}

impl<'a> DiscordIPC for Client<'a> {
    fn send(&mut self, opcode: u32, payload: &Value) -> Result<()> {
        let raw = serde_json::to_vec(payload)?;
        let mut buf = Vec::with_capacity(8 + raw.len());
        buf.write_u32::<LittleEndian>(opcode)?;
        buf.write_u32::<LittleEndian>(raw.len() as u32)?;
        buf.extend_from_slice(&raw);
        self.socket_port.write_all(&buf)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<(u32, Value)> {
        let mut header = [0u8; 8];
        self.socket_port.read_exact(&mut header)?;
        let mut rdr = &header[..];
        let op = rdr.read_u32::<LittleEndian>()?;
        let length = rdr.read_u32::<LittleEndian>()?;
        let mut data = vec![0u8; length as usize];
        self.socket_port.read_exact(&mut data)?;
        let value: Value = serde_json::from_slice(&data)?;
        Ok((op, value))
    }

    fn handshake(&mut self) -> Result<()> {
        let payload = json!({"v": 1, "client_id": self.client_id});
        self.send(0, &payload)?;
        let (_op, data) = self.recv()?;
        println!("Handshake response: {}", data);
        Ok(())
    }

    fn set_activity(&mut self, activity: Value) -> Result<()> {
        let payload = json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": process::id(),
                "activity": activity
            },
            "nonce": "set_activity_nonce"
        });
        self.send(1, &payload)?;
        let (_op, data) = self.recv()?;
        println!("Set Activity response: {}", data);
        Ok(())
    }

    fn unset_activity(&mut self) -> Result<()> {
        let payload = json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": process::id(),
                "activity": Value::Null
            },
            "nonce": "unset_activity_nonce"
        });
        self.send(1, &payload)?;
        let (_op, data) = self.recv()?;
        println!("Unset Activity response: {}", data);
        Ok(())
    }

    fn close(&mut self) {
        let _ = self.socket_port.shutdown(std::net::Shutdown::Both);
    }
}

fn main() -> anyhow::Result<()> {
    let stream = UnixStream::connect("/run/user/1000/discord-ipc-0")?;
    let client_id: &str = "1416069067697033216";
    let mut client = Client::new(&client_id, stream);

    // Perform handshake
    client.handshake()?;

    // Set activity
    let start = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    let activity = json!({
        "state": "Playing a game",
        "details": "In the menu",
        "timestamps": {"start": start},
        "assets": {
            "large_image": "your_image_key",
            "large_text": "This is a large image"
        }
    });
    client.set_activity(activity)?;

    // Keep activity for some time
    std::thread::sleep(std::time::Duration::from_secs(10));

    // Unset (clear) activity
    client.unset_activity()?;

    // Close connection
    client.close();

    Ok(())
}
