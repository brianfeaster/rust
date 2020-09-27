use ::std::fmt;
use ::std::thread;
use ::std::net::{TcpStream, TcpListener};
use ::std::sync::mpsc::{Sender};
use ::std::io::prelude::*; // OR use std::io::{Write, Read}

// Bidirectional pipe of bytes.
// When recv is called, bytes are written to buff, not aways at position 0
pub struct Ipc {
    stream: TcpStream,
    pub id: i32,
    buff: String,  // Where incoming bytes are written to always at [0..]
    stage: String, // Buffer spillover
    stagei: usize, // Staging buffer read index
}

// 
pub struct IpcMsg {
    pub id: i32, // Id of agent that sent  (or my Id when sending)
    pub msg: String // newline delimited string acquired from last recv
}

impl Ipc {

    pub fn new (stream:TcpStream, id:i32) -> Self {
        crate::Ipc{
            stream : stream,
            id     : id,
            buff   : " ".repeat(64),
            stage  : String::new(),
            stagei : 0}
    } // Ipc::new

    pub fn send (self :&mut crate::Ipc,
                 msg  :&IpcMsg) -> i32 {
        let buff = format!("{} {}\n", msg.id, msg.msg);
        match self.stream.write(buff.as_bytes()) {
            Ok(len) => len as i32,
            Err(e) => { eprintln!("{:?}", e); -1 }
        }
    } // Ipc::sendread.

    // This should snarf bytes into the local buffer until a newline is read.
    // The bytes will be copied to a new string and returned in an IpcMsg.
    //          buff       idx,stage
    //          [____]     0 []     Initial
    //          [a.__]     0 []     Emit "a"    (Read "a.", update tail index, )
    //          [b___]     0 [b]     Partial,return
    //          [cdef]     0 [bcdef] Partial continue
    //          [.f__]     0 [f]    Emit "bcdef"
    //          [g.__]     0 [f]    Emit "fg"
    //          [____]     0 []     Final
    // If last read char is newline, always reset to Initial
    pub fn recv_line (self :&mut Ipc) -> Option<String> {

        // If there's a full msg in staging, update the read index and return the msg.
        // Otherwise staging is empty or has partial message, so can be reset
        // after the next message is acquired.
        let stages :&str = &self.stage[self.stagei..];
        match stages.find("\n") {
           Some(idx) => {
               self.stagei += idx + 1;
               return Some(String::from(&stages[..=idx]));
           }
           None => 0
        };

        // Read bytes from OS into buffer
        let count_buff :usize =
            match self.stream.read( unsafe { self.buff.as_bytes_mut() } ) {
                Ok(count_buff) => {
                    //println!("\x1b[35mREAD {:#?}", count_buff);
                    count_buff
                },
                Err(e) => { 
                    //println!("\x1b[35mREAD {:?}", e);
                    if e.kind() == std::io::ErrorKind::ConnectionReset {
                        self.stream.shutdown(std::net::Shutdown::Both).expect("shutdown failed");
                    }
                    return None;
                 } // e.kind() == std::io::ErrorKind::WouldBlock
            };

        let count_msg :usize = 
            match self.buff[..count_buff].find("\n") {
               Some(idx) => idx + 1,
               None => 0
            };

        let mut new_msg :Option<String> = None;

        //println!("\x1b[36m count_buff {}  count_msg {}", count_buff, count_msg);
        if 1 <= count_msg {
            // Create new message from stage[stagei..] + buff[..count_mg]
            new_msg = Some(
                 String::from(&self.stage[self.stagei..])
                  + &self.buff[..count_msg]);
            // Reset stage
            self.stage.clear();
            self.stagei = 0;
        }
        // Append any extra buffer chars to staging
        self.stage.push_str(&self.buff[count_msg..count_buff]);
        return new_msg;
    } // Ipc::recv_line

    pub fn recv (self :&mut Ipc) -> Option<IpcMsg> {
        //// Clear receive buffer so debugging is easier
        //unsafe{self.buff.as_bytes_mut()}.iter_mut().map( |c| { *c = b' '; } ).count();
        let theline = self.recv_line();
        match theline {
            Some(msg) => {
                let (idstr, msgstr) : (&str, &str) = msg.split_at(msg.find(' ').unwrap_or(0));
                let newid = idstr.parse::<i32>().unwrap_or(-1);
                if 0 <= newid  {
                    Some(IpcMsg{  id: newid,
                                 msg: String::from(msgstr.trim())})
                } else {
                    Some(IpcMsg{  id: -1,
                                 msg: String::from(msg)})
                }
            },
            None => None
        }
    } // Ipc::recv
}

impl fmt::Debug for IpcMsg {
    fn fmt(self:&IpcMsg, f: &mut fmt::Formatter
        ) -> fmt::Result
    {
        f.debug_struct("IpcMsg")
         .field("id", &self.id)
         .field("msg",  &self.msg)
         .finish()
    }
}

// TODO: Encapsulate server and client into a single agent
// that handles connectivity to everyone else.  Star topology.

pub fn server (listener:TcpListener,  txipc:Sender<Ipc>) {
    thread::spawn( move || {
      // I'm the server.  So wait for a connection, then create the Ipc object for myself.
      // Also notify the client it's player 1 (I'm 0)
      let mut playerCount = 0;
      for stream in listener.incoming() {
          let mut ipc = Ipc::new(stream.unwrap(), playerCount);
          ipc.stream.set_read_timeout(
               std::option::Option::Some(
                   ::std::time::Duration::new(0, 1))).expect("ERROR: set_read_timeout");
          // Broadcast first msg to new client/player: uniqueID and game type
          playerCount += 1;
          let msg :IpcMsg = IpcMsg{id:playerCount, msg:"RUSTEROIDS".to_string()};
          let len :i32    = ipc.send(&msg);
          println!("[{}]> {:?} {}", ipc.id, msg, len);
          txipc.send(ipc).expect("Server unable to send Ipc to local channel.");
        }
      }); // thread lambda
}


// Send the Ipc object through the channel.  The agent will use this to communicate with the remote agent.
pub fn client (txipc: Sender<Ipc>) {
    let stream = TcpStream::connect("127.0.0.1:7145").unwrap();
    let mut ipc = Ipc::new(stream, -1);
    let msg0 = ipc.recv().unwrap();

    ipc.id = msg0.id;
    println!(">[{}] {:?}", ipc.id, msg0);

    ipc.stream.set_read_timeout(
         std::option::Option::Some(
             ::std::time::Duration::new(0, 1))).expect("ERROR: set_read_timeout");

    txipc.send(ipc).expect("Client unable to send Ipc to local channel.");
}