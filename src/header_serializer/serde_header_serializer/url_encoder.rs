/// custom URL encoder based on [backblaze url encoding](https://www.backblaze.com/b2/docs/string_encoding.html)
/// urlencoder crate can not be used, as for example it does encode '/' which prevents b2 from realizing it as a folder separator

const ALLOWED_SPECIAL_CHARS: &'static str = "._-/~!$'()*;=:@";
pub fn url_encode(s: &str) -> String {
    let mut res_elems = Vec::with_capacity(s.as_bytes().len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || ALLOWED_SPECIAL_CHARS.contains(c) {
            res_elems.push(String::from(c));
        } else {
            let mut buf = [0u8; 4];
            for byte in c.encode_utf8(&mut buf).as_bytes() {
                res_elems.push(format!("%{:02X}", byte));
            }
        }
    }
    res_elems.concat()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        for test_case in TEST_DATA_SETS {
            let encoded = url_encode(test_case.string);
            let expectations = [test_case.minimally_encoded, test_case.fully_encoded];
            assert!(
                expectations.contains(&encoded.as_str()),
                "Encoded value {:#?} for {:#?} was not in expectations: {:#?}",
                encoded,
                test_case.string,
                expectations
            );
        }
    }

    struct TestData {
        fully_encoded: &'static str,
        minimally_encoded: &'static str,
        string: &'static str,
    }

    const TEST_DATA_SETS: &[TestData] = &[
        TestData {
            fully_encoded: "%20",
            minimally_encoded: "+",
            string: " ",
        },
        TestData {
            fully_encoded: "%21",
            minimally_encoded: "!",
            string: "!",
        },
        TestData {
            fully_encoded: "%22",
            minimally_encoded: "%22",
            string: "\"",
        },
        TestData {
            fully_encoded: "%23",
            minimally_encoded: "%23",
            string: "#",
        },
        TestData {
            fully_encoded: "%24",
            minimally_encoded: "$",
            string: "$",
        },
        TestData {
            fully_encoded: "%25",
            minimally_encoded: "%25",
            string: "%",
        },
        TestData {
            fully_encoded: "%26",
            minimally_encoded: "%26",
            string: "&",
        },
        TestData {
            fully_encoded: "%27",
            minimally_encoded: "'",
            string: "'",
        },
        TestData {
            fully_encoded: "%28",
            minimally_encoded: "(",
            string: "(",
        },
        TestData {
            fully_encoded: "%29",
            minimally_encoded: ")",
            string: ")",
        },
        TestData {
            fully_encoded: "%2A",
            minimally_encoded: "*",
            string: "*",
        },
        TestData {
            fully_encoded: "%2B",
            minimally_encoded: "%2B",
            string: "+",
        },
        TestData {
            fully_encoded: "%2C",
            minimally_encoded: "%2C",
            string: ",",
        },
        TestData {
            fully_encoded: "%2D",
            minimally_encoded: "-",
            string: "-",
        },
        TestData {
            fully_encoded: "%2E",
            minimally_encoded: ".",
            string: ".",
        },
        TestData {
            fully_encoded: "/",
            minimally_encoded: "/",
            string: "/",
        },
        TestData {
            fully_encoded: "%30",
            minimally_encoded: "0",
            string: "0",
        },
        TestData {
            fully_encoded: "%31",
            minimally_encoded: "1",
            string: "1",
        },
        TestData {
            fully_encoded: "%32",
            minimally_encoded: "2",
            string: "2",
        },
        TestData {
            fully_encoded: "%33",
            minimally_encoded: "3",
            string: "3",
        },
        TestData {
            fully_encoded: "%34",
            minimally_encoded: "4",
            string: "4",
        },
        TestData {
            fully_encoded: "%35",
            minimally_encoded: "5",
            string: "5",
        },
        TestData {
            fully_encoded: "%36",
            minimally_encoded: "6",
            string: "6",
        },
        TestData {
            fully_encoded: "%37",
            minimally_encoded: "7",
            string: "7",
        },
        TestData {
            fully_encoded: "%38",
            minimally_encoded: "8",
            string: "8",
        },
        TestData {
            fully_encoded: "%39",
            minimally_encoded: "9",
            string: "9",
        },
        TestData {
            fully_encoded: "%3A",
            minimally_encoded: ":",
            string: ":",
        },
        TestData {
            fully_encoded: "%3B",
            minimally_encoded: ";",
            string: ";",
        },
        TestData {
            fully_encoded: "%3C",
            minimally_encoded: "%3C",
            string: "<",
        },
        TestData {
            fully_encoded: "%3D",
            minimally_encoded: "=",
            string: "=",
        },
        TestData {
            fully_encoded: "%3E",
            minimally_encoded: "%3E",
            string: ">",
        },
        TestData {
            fully_encoded: "%3F",
            minimally_encoded: "%3F",
            string: "?",
        },
        TestData {
            fully_encoded: "%40",
            minimally_encoded: "@",
            string: "@",
        },
        TestData {
            fully_encoded: "%41",
            minimally_encoded: "A",
            string: "A",
        },
        TestData {
            fully_encoded: "%42",
            minimally_encoded: "B",
            string: "B",
        },
        TestData {
            fully_encoded: "%43",
            minimally_encoded: "C",
            string: "C",
        },
        TestData {
            fully_encoded: "%44",
            minimally_encoded: "D",
            string: "D",
        },
        TestData {
            fully_encoded: "%45",
            minimally_encoded: "E",
            string: "E",
        },
        TestData {
            fully_encoded: "%46",
            minimally_encoded: "F",
            string: "F",
        },
        TestData {
            fully_encoded: "%47",
            minimally_encoded: "G",
            string: "G",
        },
        TestData {
            fully_encoded: "%48",
            minimally_encoded: "H",
            string: "H",
        },
        TestData {
            fully_encoded: "%49",
            minimally_encoded: "I",
            string: "I",
        },
        TestData {
            fully_encoded: "%4A",
            minimally_encoded: "J",
            string: "J",
        },
        TestData {
            fully_encoded: "%4B",
            minimally_encoded: "K",
            string: "K",
        },
        TestData {
            fully_encoded: "%4C",
            minimally_encoded: "L",
            string: "L",
        },
        TestData {
            fully_encoded: "%4D",
            minimally_encoded: "M",
            string: "M",
        },
        TestData {
            fully_encoded: "%4E",
            minimally_encoded: "N",
            string: "N",
        },
        TestData {
            fully_encoded: "%4F",
            minimally_encoded: "O",
            string: "O",
        },
        TestData {
            fully_encoded: "%50",
            minimally_encoded: "P",
            string: "P",
        },
        TestData {
            fully_encoded: "%51",
            minimally_encoded: "Q",
            string: "Q",
        },
        TestData {
            fully_encoded: "%52",
            minimally_encoded: "R",
            string: "R",
        },
        TestData {
            fully_encoded: "%53",
            minimally_encoded: "S",
            string: "S",
        },
        TestData {
            fully_encoded: "%54",
            minimally_encoded: "T",
            string: "T",
        },
        TestData {
            fully_encoded: "%55",
            minimally_encoded: "U",
            string: "U",
        },
        TestData {
            fully_encoded: "%56",
            minimally_encoded: "V",
            string: "V",
        },
        TestData {
            fully_encoded: "%57",
            minimally_encoded: "W",
            string: "W",
        },
        TestData {
            fully_encoded: "%58",
            minimally_encoded: "X",
            string: "X",
        },
        TestData {
            fully_encoded: "%59",
            minimally_encoded: "Y",
            string: "Y",
        },
        TestData {
            fully_encoded: "%5A",
            minimally_encoded: "Z",
            string: "Z",
        },
        TestData {
            fully_encoded: "%5B",
            minimally_encoded: "%5B",
            string: "[",
        },
        TestData {
            fully_encoded: "%5C",
            minimally_encoded: "%5C",
            string: "\\",
        },
        TestData {
            fully_encoded: "%5D",
            minimally_encoded: "%5D",
            string: "]",
        },
        TestData {
            fully_encoded: "%5E",
            minimally_encoded: "%5E",
            string: "^",
        },
        TestData {
            fully_encoded: "%5F",
            minimally_encoded: "_",
            string: "_",
        },
        TestData {
            fully_encoded: "%60",
            minimally_encoded: "%60",
            string: "`",
        },
        TestData {
            fully_encoded: "%61",
            minimally_encoded: "a",
            string: "a",
        },
        TestData {
            fully_encoded: "%62",
            minimally_encoded: "b",
            string: "b",
        },
        TestData {
            fully_encoded: "%63",
            minimally_encoded: "c",
            string: "c",
        },
        TestData {
            fully_encoded: "%64",
            minimally_encoded: "d",
            string: "d",
        },
        TestData {
            fully_encoded: "%65",
            minimally_encoded: "e",
            string: "e",
        },
        TestData {
            fully_encoded: "%66",
            minimally_encoded: "f",
            string: "f",
        },
        TestData {
            fully_encoded: "%67",
            minimally_encoded: "g",
            string: "g",
        },
        TestData {
            fully_encoded: "%68",
            minimally_encoded: "h",
            string: "h",
        },
        TestData {
            fully_encoded: "%69",
            minimally_encoded: "i",
            string: "i",
        },
        TestData {
            fully_encoded: "%6A",
            minimally_encoded: "j",
            string: "j",
        },
        TestData {
            fully_encoded: "%6B",
            minimally_encoded: "k",
            string: "k",
        },
        TestData {
            fully_encoded: "%6C",
            minimally_encoded: "l",
            string: "l",
        },
        TestData {
            fully_encoded: "%6D",
            minimally_encoded: "m",
            string: "m",
        },
        TestData {
            fully_encoded: "%6E",
            minimally_encoded: "n",
            string: "n",
        },
        TestData {
            fully_encoded: "%6F",
            minimally_encoded: "o",
            string: "o",
        },
        TestData {
            fully_encoded: "%70",
            minimally_encoded: "p",
            string: "p",
        },
        TestData {
            fully_encoded: "%71",
            minimally_encoded: "q",
            string: "q",
        },
        TestData {
            fully_encoded: "%72",
            minimally_encoded: "r",
            string: "r",
        },
        TestData {
            fully_encoded: "%73",
            minimally_encoded: "s",
            string: "s",
        },
        TestData {
            fully_encoded: "%74",
            minimally_encoded: "t",
            string: "t",
        },
        TestData {
            fully_encoded: "%75",
            minimally_encoded: "u",
            string: "u",
        },
        TestData {
            fully_encoded: "%76",
            minimally_encoded: "v",
            string: "v",
        },
        TestData {
            fully_encoded: "%77",
            minimally_encoded: "w",
            string: "w",
        },
        TestData {
            fully_encoded: "%78",
            minimally_encoded: "x",
            string: "x",
        },
        TestData {
            fully_encoded: "%79",
            minimally_encoded: "y",
            string: "y",
        },
        TestData {
            fully_encoded: "%7A",
            minimally_encoded: "z",
            string: "z",
        },
        TestData {
            fully_encoded: "%7B",
            minimally_encoded: "%7B",
            string: "{",
        },
        TestData {
            fully_encoded: "%7C",
            minimally_encoded: "%7C",
            string: "|",
        },
        TestData {
            fully_encoded: "%7D",
            minimally_encoded: "%7D",
            string: "}",
        },
        TestData {
            fully_encoded: "%7E",
            minimally_encoded: "~",
            string: "~",
        },
        TestData {
            fully_encoded: "%7F",
            minimally_encoded: "%7F",
            string: "\u{007f}",
        },
        TestData {
            fully_encoded: "%E8%87%AA%E7%94%B1",
            minimally_encoded: "%E8%87%AA%E7%94%B1",
            string: "\u{81ea}\u{7531}",
        },
        // TODO: how to represent this in rust:
        //  TestData{fullyEncoded: "%F0%90%90%80", minimallyEncoded: "%F0%90%90%80", string: "\ud801\udc00"}
    ];
}
