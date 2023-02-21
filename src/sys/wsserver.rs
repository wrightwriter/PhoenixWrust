use std::{time::{Duration, SystemTime}, sync::{mpsc::channel, Arc, Mutex}, borrow::BorrowMut, collections::HashMap};

use enum_variants_strings::EnumVariantsStrings;
use futures::{StreamExt, pin_mut, SinkExt, stream::SplitSink};
use nalgebra_glm::{lerp, vec1};
use serde_json::{Map, Value};
use smallvec::SmallVec;
use strum::EnumIter;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};

use tungstenite::{WebSocket, Message};

use super::wtime::WTime;


#[derive(Debug, EnumIter,EnumVariantsStrings)]
#[enum_variants_strings_transform(transform = "none")]
#[allow(non_camel_case_types)]
pub enum WChannel {
    amogus,
    bamonge
}

#[derive(Debug, Clone)]
pub enum WsMessage {
  PLAY,
  PAUSE,
  SEEK(f64),
  DATA(Map<String, Value>),
  NONE
}

pub struct WsServer{
    pub messages_to_send: Arc<Mutex<SmallVec<[WsMessage;32]>>>,
    pub messages_to_receive: Arc<Mutex<SmallVec<[WsMessage;32]>>>,
    pub ws_write:  SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub data: Vec<f64>
}

impl WsServer{
    pub fn get_channel(&self, chan: WChannel)->f64{
        self.data[chan as u32 as usize]
    }
    pub fn set_channel(&mut self, chan: WChannel, val: f64){
        self.data[chan as u32 as usize] = val;
    }
    pub async fn tick(&mut self, w_time: &mut WTime){
        profiling::scope!("tick server");
        let mut messages_to_receive: SmallVec<[WsMessage;32]> = SmallVec::new();
        
        unsafe {
            let mut messages = self.messages_to_receive.lock().unwrap();
            for message in messages.iter() {
                messages_to_receive.push(message.clone());
            }
            messages.set_len(0);
        }
        
        for message in messages_to_receive.iter() {
            match message {
                WsMessage::PLAY => {
                    w_time.play();
                },
                WsMessage::PAUSE => {
                    wprint!("---- PAUSED");
                    w_time.pause();
                },
                WsMessage::SEEK(t) => {
                    w_time.set_time(*t);
                },
                WsMessage::NONE => {
                },
                WsMessage::DATA(data) => {
                    for d in data {
                        let chan = WChannel::from_str(d.0).unwrap();
                        self.set_channel(chan, d.1.as_f64().unwrap());
                    }
                },
            }
        }

        let t = w_time.t_f32;
        
        {
          profiling::scope!("send ws time");
          let mut ms = r#"{
                      "type": "update",
                      "time": "#.to_owned();
          ms.push_str(&t.to_string());

          ms += r#" }"#;
          ms += "\n";

          self.ws_write.send(
              tungstenite::Message::Text( ms)
          ).await.unwrap();
        }
    }

    pub async fn new()->Self{
        let messages_to_send = Arc::new(Mutex::new(SmallVec::new()));
        let messages_to_send_clone = messages_to_send.clone();

        let messages_to_receive = Arc::new(Mutex::new(SmallVec::new()));
        let messages_to_receive_clone = messages_to_receive.clone();
            
        let addr = url::Url::parse("ws://localhost:12250/");

        let (mut ws_stream, _) = tokio_tungstenite::connect_async("ws://localhost:12250/").await.unwrap();

        
        let (mut ws_write, mut ws_read) = ws_stream.split();

        {
          profiling::scope!("send ws time");
          ws_write.send(
            //   tungstenite::Message::Text("amogus".to_string())
              tungstenite::Message::Text(
                  r#"{
                      "type": "update",
                      "time": 0.1
                  }"#.to_owned() +"\n"
              )
          ).await.unwrap();
        }

        
        tokio::spawn(
        async move{
            loop {
                let msg = ws_read.next().await.unwrap().unwrap();
                let msg = msg.to_string();
                let msg: serde_json::Value = serde_json::from_str(&msg).unwrap();
                let ty = msg.get("type").unwrap().as_str().unwrap();
                let ws_message = match ty {
                    "pause" => {
                        println!("{}",msg);
                        WsMessage::PAUSE
                    },
                    "play" => {
                        println!("{}",msg);
                        WsMessage::PLAY
                    },
                    "seek" => {
                        println!("{}",msg);
                        let time = msg.get("time").unwrap().as_f64().unwrap();
                        WsMessage::SEEK(time)
                    },
                    "data" => {
                        let data = msg.get("data").unwrap().as_object().unwrap();
                        WsMessage::DATA(data.clone())
                    },
                    &_ => {
                        WsMessage::NONE
                    }
                };
                {
                    let mut msg_to_receive = messages_to_receive_clone.lock().unwrap();
                    msg_to_receive.push(ws_message);
                }
            } 
        });
        

        wprint!("--- LISTENING ---");

        wprint!("!!! STREAMING !!!");
        
        let data: Vec<f64>  = (0..100).map(|_| 100.0).collect();
        
        Self {
            messages_to_send,
            messages_to_receive,
            ws_write,
            data: data,
        }
    }
    
}
