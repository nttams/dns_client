// todo: Request and Response should share a same trait? UPDATE: really?
use super::header::Header;
use super::question::Question;
use super::record::*;
use crate::config::config_handler::CONFIG;

pub struct Message {
    header: Header,
    question: Question,
    answers: Records,
    authorities: Records,
    additionals: Records,
}

impl Message {
    pub fn new(domain_name: &str) -> Self {
        let mut msg = Self {
            header: Header::new(),
            question: Question::new(&domain_name),
            answers: Records::new(),
            authorities: Records::new(),
            additionals: Records::new(),
        };

        msg.header.inc_qd_count();
        msg.header.enable_recursion();
        msg
    }

    pub fn convert_to_edns_query(&mut self) {
        let opt_rr = OptRecord::new(CONFIG.edns_buffer_size as u16);
        self.add_additional_records(opt_rr.record);
    }

    pub fn add_2_bytes_length_for_tcp(msg: &mut Vec<u8>) {
        let len = msg.len();
        msg.insert(0, ((len >> 0) & 0b0000_0000_1111_1111) as u8);
        msg.insert(0, ((len >> 8) & 0b0000_0000_1111_1111) as u8);
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();

        let mut encoded_header = self.header.encode();
        let mut encoded_questions = self.question.encode();
        let mut encoded_answers = self.answers.encode();
        let mut encoded_authorities = self.authorities.encode();
        let mut encoded_additionals = self.additionals.encode();

        result.append(&mut encoded_header);
        result.append(&mut encoded_questions);
        result.append(&mut encoded_answers);
        result.append(&mut encoded_authorities);
        result.append(&mut encoded_additionals);

        result
    }

    pub fn parse(msg: &[u8]) -> Self {
        let mut pos = 0;

        let (header, parsed_count) = Header::parse(msg, pos);
        pos += parsed_count as usize;

        let (question, parsed_count) = Question::parse(msg, pos);
        pos += parsed_count as usize;

        let mut answers = Records::new();

        for _ in 0..header.get_an_count() {
            let (answer, parsed_count) = Record::parse(msg, pos);
            pos += parsed_count as usize;

            answers.push(answer);
        }

        let mut authorities = Records::new();
        for _ in 0..header.get_ns_count() {
            let (authority, parsed_count) = Record::parse(msg, pos);
            pos += parsed_count as usize;

            authorities.push(authority);
        }

        let mut additionals = Records::new();
        for _ in 0..header.get_ar_count() {
            let (additional, parsed_count) = Record::parse(msg, pos);
            pos += parsed_count as usize;

            additionals.push(additional);
        }

        Self {
            header,
            question,
            answers,
            authorities,
            additionals,
        }
    }

    pub fn get_answers(&self) -> &Records {
        &self.answers
    }

    pub fn add_additional_records(&mut self, record: Record) {
        self.additionals.push(record);
        self.header.inc_ar_count();
    }

    // quick check if tc bit is ON, no need to parse
    // todo: not tested
    pub fn is_truncated(raw_msg: &Vec<u8>) -> bool {
        let is_truncated = raw_msg[3] & 0b00000010;
        is_truncated == 1
    }
}
