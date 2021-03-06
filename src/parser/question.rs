use super::dns_types;
use super::dns_types::{QClass, QType};
use super::domain_name::DomainName;

pub struct Question {
    // more than one domain can exist at a same time,
    // but for simplicity, only one is allowed.
    entry: QuestionEntry,
}

impl Question {
    pub fn new(domain_name: &str) -> Self {
        Self {
            entry: QuestionEntry::new(&domain_name),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        self.entry.encode()
    }

    pub fn parse(msg: &[u8], pos: usize) -> (Self, u16) {
        let (entry, parsed_count) = QuestionEntry::parse(msg, pos);
        let question = Self { entry };
        (question, parsed_count)
    }
}

struct QuestionEntry {
    q_name: DomainName,
    q_type: u16,
    q_class: u16,
}

impl QuestionEntry {
    fn new(domain_name: &str) -> Self {
        assert!(domain_name.len() < 256);

        Self {
            q_name: DomainName::new(&domain_name),
            q_type: QType::get_value(QType::A).expect("100% sure"),
            q_class: QClass::get_value(QClass::In).expect("100% sure"),
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut result = self.q_name.encode();
        let mut type_and_class = dns_types::encode_type_and_class(self.q_type, self.q_class);

        result.append(&mut type_and_class);
        result
    }

    fn parse(msg: &[u8], pos: usize) -> (Self, u16) {
        assert!(msg.len() > 4);

        let mut pos_mut = pos;

        let (q_name, parsed_count) = DomainName::parse(msg, pos_mut);
        pos_mut += parsed_count as usize;

        let (q_type, q_class, parsed_count) = dns_types::parse_type_and_class(msg, pos_mut);
        pos_mut += parsed_count as usize;

        let parsed_count: u16 = (pos_mut - pos)
            .try_into()
            .expect("can not happen ever as msg length is controlled");

        (
            Self {
                q_name,
                q_type,
                q_class,
            },
            parsed_count,
        )
    }
}
