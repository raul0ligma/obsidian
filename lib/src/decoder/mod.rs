use crate::verifier::{Node, NodeType};
pub struct NodeDecoder;

impl NodeDecoder {
    pub fn decode_inner(input: &[u8], input_offset: usize) -> (Vec<u8>, usize) {
        let mut offset = input_offset;
        let mut out: Vec<u8> = Vec::new();

        // always assume the encoded value will be a string
        // hence would be between 0x80 and 0xbf
        let item_type = input[offset];
        if item_type < 0x80 {
            out = vec![item_type];
            offset += 1;
        } else if item_type <= 0xb7 {
            // read length encoded in 1 byte
            let actual_length = (item_type - 0x80) as usize;
            // read length
            offset += 1;
            out = input[offset..offset + actual_length].to_vec();
            // shift the len
            offset += actual_length;
        } else if item_type <= 0xbf {
            let length_bytes = (item_type - 0xb7) as usize;
            offset += 1;
            // read in the first value and set it in a usize
            let mut actual_length = (input[offset]) as usize;

            // read the whole length if needed now
            if length_bytes > 1 {
                for i in 1..=length_bytes - 1 {
                    actual_length = (actual_length << 8) | (input[offset + i]) as usize;
                }
            }

            // shift the read length
            offset += length_bytes;

            out = input[offset..offset + actual_length].to_vec();

            offset += actual_length;
        } else {
            // do not handle anything else
            panic!("too much work, use a lib at this point bro");
        }
        (out, offset)
    }

    pub fn decode_rlp(input: &[u8]) -> Vec<Vec<u8>> {
        // only handle lists
        if input[0] < 0xf7 {
            let (out, offset) = Self::decode_inner(input, 0);
            if offset < input.len() {
                panic!(
                    "could not decode full expected {} found {}",
                    offset,
                    input.len()
                )
            }
            return vec![out];
        }
        // we skip reading length and directly shift offset to data as the given node is always a complete list
        let mut offset: usize = 1usize + (input[0] - 0xf7) as usize;

        let mut out: Vec<Vec<u8>> = Vec::new();
        while offset < input.len() {
            let (parsed, new_offset) = Self::decode_inner(input, offset);

            out.push(parsed);
            offset = new_offset;
        }
        out
    }

    pub fn decode_mpt_node(input: &[u8]) -> Node {
        let decoded = NodeDecoder::decode_rlp(input);
        match decoded.len() {
            17 => Node {
                original: input.to_vec(),
                node: NodeType::Branch(decoded),
            },
            2 => {
                // 0000 Extension Even
                // 0001 Extension OddE
                // 0010 Leaf Even
                // 0011 Leaf Odd

                // find the first nibble
                let prefix = decoded[0][0] >> 4;

                if (prefix & 0x2) != 0 {
                    Node {
                        original: input.to_vec(),
                        node: NodeType::Leaf(
                            // true if odd
                            prefix & 0x1 != 0,
                            decoded[0].clone(),
                            decoded[1].clone(),
                        ),
                    }
                } else {
                    Node {
                        original: input.to_vec(),
                        node: NodeType::Extension(
                            // true if odd
                            prefix & 0x1 != 0,
                            decoded[0].clone(),
                            decoded[1].clone(),
                        ),
                    }
                }
            }
            _ => {
                panic!("we don't support this node");
            }
        }
    }
}
