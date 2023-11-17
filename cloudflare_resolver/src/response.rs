use std::collections::HashMap;
use std::net::IpAddr;

use serde::Deserialize;

const A: u16 = 1;
const CNAME: u16 = 5;
const AAAA: u16 = 28;

#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    #[serde(rename = "Question")]
    question: Vec<Question>,
    #[serde(rename = "Answer")]
    answer: Vec<Answer>,
}

#[derive(Debug, Clone, Deserialize)]
struct Question {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Answer {
    name: String,
    #[serde(rename = "type")]
    dns_type: u16,
    #[serde(rename = "TTL")]
    ttl: u32,
    data: String,
}

impl Response {
    pub fn resolve(&self) -> (Vec<IpAddr>, u32) {
        let mut names = HashMap::with_capacity(self.answer.len());
        for answer in &self.answer {
            let name = answer.name.as_str();
            if !names.contains_key(name) {
                names.insert(name, Vec::new());
            }
            names.get_mut(name).unwrap().push(answer);
        }

        let mut stack = Vec::with_capacity(names.len());
        for question in &self.question {
            stack.push(question.name.as_str());
        }

        let mut ttl = u32::MAX;
        let mut addrs = Vec::with_capacity(names.len());
        while let Some(name) = stack.pop() {
            let Some(answers) = names.remove(name) else {
                eprintln!("No name found for {name} in {names:?}");
                continue;
            };
            for answer in answers {
                match answer.dns_type {
                    CNAME => {
                        stack.push(answer.data.trim_end_matches('.')); // DFS the cname
                    }
                    A | AAAA => match answer.data.parse::<IpAddr>() {
                        Ok(addr) => {
                            ttl = ttl.min(answer.ttl);
                            addrs.push(addr);
                        }
                        Err(_) => {
                            eprintln!("IpAddr parsing error {answer:?}");
                        }
                    },
                    _ => { /* I don't know what to do */ }
                }
            }
        }
        (addrs, ttl)
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;

    #[test]
    fn test_socket_parsing() {
        let addr = "2a03:2880:f16b:81:face:b00c:0:25de".parse::<IpAddr>();
        println!("{addr:?}");

        let addr = SocketAddr::from_str("2600:9000:2654:7c00:7:49a5:5fd2:8621");
        println!("{addr:?}");

        let addr = SocketAddr::from_str("127.0.0.1");
        println!("{addr:?}");
    }
}
