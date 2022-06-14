pub enum DomainName {
    LiteralDomainName(String),
    LabelToDomainName(u16),
}

const LENGTH_OF_LABEL_DOMAIN: u16 = 2;

impl DomainName {
    pub fn new(domain_name: &str) -> Self {
        // this will create a new String
        Self::LiteralDomainName(String::from(domain_name))
    }

    pub fn encode(&self) -> Vec<u8> {
        match &self {
            Self::LiteralDomainName(domain_name) => Self::encode_literal_domain(&domain_name),
            Self::LabelToDomainName(position) => Self::encode_pointer_domain(*position),
        }
    }

    // todo: ugly function, improve it
    fn encode_literal_domain(domain_name: &str) -> Vec<u8> {
        // root domain, for OPT record
        if domain_name.len() == 1 && domain_name.as_bytes()[0] == 0 {
            return vec![0];
        }

        let mut q_name = String::from(".");
        q_name.push_str(domain_name);

        let mut q_name_as_bytes = q_name.into_bytes();

        let mut dot_points = Vec::<u8>::new();

        for (i, &item) in q_name_as_bytes.iter().enumerate() {
            if item == b'.' {
                dot_points.push(i as u8);
            }
        }

        for i in 0..(dot_points.len() - 1) {
            q_name_as_bytes[dot_points[i as usize] as usize] =
                dot_points[(i + 1) as usize] - dot_points[i as usize] - 1;
        }

        q_name_as_bytes[dot_points[dot_points.len() - 1 as usize] as usize] =
            q_name_as_bytes.len() as u8 - dot_points[dot_points.len() - 1 as usize] as u8 - 1;

        // null termination
        q_name_as_bytes.push(0);
        q_name_as_bytes
    }

    // todo: add a helper to encode u16, u32 to Vec<u8>
    fn encode_pointer_domain(position: u16) -> Vec<u8> {
        let mut result: Vec<u8> = vec![0; 2];
        result[0] = 0b1100_0000;
        result[0] |= (position >> 8) as u8;
        result[1] |= (position >> 0) as u8;

        result
    }

    pub fn parse(msg: &[u8], offset: usize) -> (Self, u16) {
        if Self::is_pointer_domain_name(msg[offset]) {
            Self::parse_pointer_domain_name(msg, offset)
        } else {
            Self::parse_literal_domain_name(msg, offset)
        }
    }

    fn parse_pointer_domain_name(msg: &[u8], offset: usize) -> (Self, u16) {
        let mut position: u16 = 0;

        position |= (msg[offset + 0] as u16) << 8;
        position |= (msg[offset + 1] as u16) << 0;
        position &= 0b0011_1111_1111_1111;

        (Self::LabelToDomainName(position), LENGTH_OF_LABEL_DOMAIN)
    }

    // todo: does not work when there's no dot in domain name
    fn parse_literal_domain_name(msg: &[u8], offset: usize) -> (Self, u16) {
        let (mut question, parsed_count) = Self::extract_domain_name_field(msg, offset);

        // todo: quick fix
        if parsed_count == 1 {
            let domain_name = Self::new("\0");
            return (domain_name, 1);
        }

        let mut pos = 0;
        loop {
            let temp = question[pos];
            question[pos] = b'.';
            pos += temp as usize + 1;
            if question[pos] == 0 {
                break;
            }
        }

        question.remove(pos);
        question.remove(0);

        let domain_name_str = String::from_utf8(question).expect("trust me :D");
        let domain_name = Self::new(&domain_name_str);

        (domain_name, parsed_count)
    }

    fn extract_domain_name_field(msg: &[u8], offset: usize) -> (Vec<u8>, u16) {
        // find the end of question field. it ends by a value 0
        // todo: does this cover all cases??
        let mut probe = offset;
        loop {
            if msg[probe] == 0 {
                probe += 1;
                break;
            } else {
                probe += 1;
            }
        }

        // it's ok to clone here as we need new value for question
        let mut question = Vec::new();
        question.extend_from_slice(&msg[offset..probe]);

        let parsed_count: u16 = (probe - offset)
            .try_into()
            .expect("can not happen ever as msg length is controlled");

        (question, parsed_count)
    }

    fn is_pointer_domain_name(msg: u8) -> bool {
        (msg & 0b1100_0000) == 0b1100_0000
    }
}
