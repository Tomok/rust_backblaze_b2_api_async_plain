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
            let expectations = [test_case.minimallyEncoded, test_case.fullyEncoded];
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
        fullyEncoded: &'static str,
        minimallyEncoded: &'static str,
        string: &'static str,
    }

    const TEST_DATA_SETS: &[TestData] = &[
        TestData {
            fullyEncoded: "%20",
            minimallyEncoded: "+",
            string: " ",
        },
        TestData {
            fullyEncoded: "%21",
            minimallyEncoded: "!",
            string: "!",
        },
        TestData {
            fullyEncoded: "%22",
            minimallyEncoded: "%22",
            string: "\"",
        },
        TestData {
            fullyEncoded: "%23",
            minimallyEncoded: "%23",
            string: "#",
        },
        TestData {
            fullyEncoded: "%24",
            minimallyEncoded: "$",
            string: "$",
        },
        TestData {
            fullyEncoded: "%25",
            minimallyEncoded: "%25",
            string: "%",
        },
        TestData {
            fullyEncoded: "%26",
            minimallyEncoded: "%26",
            string: "&",
        },
        TestData {
            fullyEncoded: "%27",
            minimallyEncoded: "'",
            string: "'",
        },
        TestData {
            fullyEncoded: "%28",
            minimallyEncoded: "(",
            string: "(",
        },
        TestData {
            fullyEncoded: "%29",
            minimallyEncoded: ")",
            string: ")",
        },
        TestData {
            fullyEncoded: "%2A",
            minimallyEncoded: "*",
            string: "*",
        },
        TestData {
            fullyEncoded: "%2B",
            minimallyEncoded: "%2B",
            string: "+",
        },
        TestData {
            fullyEncoded: "%2C",
            minimallyEncoded: "%2C",
            string: ",",
        },
        TestData {
            fullyEncoded: "%2D",
            minimallyEncoded: "-",
            string: "-",
        },
        TestData {
            fullyEncoded: "%2E",
            minimallyEncoded: ".",
            string: ".",
        },
        TestData {
            fullyEncoded: "/",
            minimallyEncoded: "/",
            string: "/",
        },
        TestData {
            fullyEncoded: "%30",
            minimallyEncoded: "0",
            string: "0",
        },
        TestData {
            fullyEncoded: "%31",
            minimallyEncoded: "1",
            string: "1",
        },
        TestData {
            fullyEncoded: "%32",
            minimallyEncoded: "2",
            string: "2",
        },
        TestData {
            fullyEncoded: "%33",
            minimallyEncoded: "3",
            string: "3",
        },
        TestData {
            fullyEncoded: "%34",
            minimallyEncoded: "4",
            string: "4",
        },
        TestData {
            fullyEncoded: "%35",
            minimallyEncoded: "5",
            string: "5",
        },
        TestData {
            fullyEncoded: "%36",
            minimallyEncoded: "6",
            string: "6",
        },
        TestData {
            fullyEncoded: "%37",
            minimallyEncoded: "7",
            string: "7",
        },
        TestData {
            fullyEncoded: "%38",
            minimallyEncoded: "8",
            string: "8",
        },
        TestData {
            fullyEncoded: "%39",
            minimallyEncoded: "9",
            string: "9",
        },
        TestData {
            fullyEncoded: "%3A",
            minimallyEncoded: ":",
            string: ":",
        },
        TestData {
            fullyEncoded: "%3B",
            minimallyEncoded: ";",
            string: ";",
        },
        TestData {
            fullyEncoded: "%3C",
            minimallyEncoded: "%3C",
            string: "<",
        },
        TestData {
            fullyEncoded: "%3D",
            minimallyEncoded: "=",
            string: "=",
        },
        TestData {
            fullyEncoded: "%3E",
            minimallyEncoded: "%3E",
            string: ">",
        },
        TestData {
            fullyEncoded: "%3F",
            minimallyEncoded: "%3F",
            string: "?",
        },
        TestData {
            fullyEncoded: "%40",
            minimallyEncoded: "@",
            string: "@",
        },
        TestData {
            fullyEncoded: "%41",
            minimallyEncoded: "A",
            string: "A",
        },
        TestData {
            fullyEncoded: "%42",
            minimallyEncoded: "B",
            string: "B",
        },
        TestData {
            fullyEncoded: "%43",
            minimallyEncoded: "C",
            string: "C",
        },
        TestData {
            fullyEncoded: "%44",
            minimallyEncoded: "D",
            string: "D",
        },
        TestData {
            fullyEncoded: "%45",
            minimallyEncoded: "E",
            string: "E",
        },
        TestData {
            fullyEncoded: "%46",
            minimallyEncoded: "F",
            string: "F",
        },
        TestData {
            fullyEncoded: "%47",
            minimallyEncoded: "G",
            string: "G",
        },
        TestData {
            fullyEncoded: "%48",
            minimallyEncoded: "H",
            string: "H",
        },
        TestData {
            fullyEncoded: "%49",
            minimallyEncoded: "I",
            string: "I",
        },
        TestData {
            fullyEncoded: "%4A",
            minimallyEncoded: "J",
            string: "J",
        },
        TestData {
            fullyEncoded: "%4B",
            minimallyEncoded: "K",
            string: "K",
        },
        TestData {
            fullyEncoded: "%4C",
            minimallyEncoded: "L",
            string: "L",
        },
        TestData {
            fullyEncoded: "%4D",
            minimallyEncoded: "M",
            string: "M",
        },
        TestData {
            fullyEncoded: "%4E",
            minimallyEncoded: "N",
            string: "N",
        },
        TestData {
            fullyEncoded: "%4F",
            minimallyEncoded: "O",
            string: "O",
        },
        TestData {
            fullyEncoded: "%50",
            minimallyEncoded: "P",
            string: "P",
        },
        TestData {
            fullyEncoded: "%51",
            minimallyEncoded: "Q",
            string: "Q",
        },
        TestData {
            fullyEncoded: "%52",
            minimallyEncoded: "R",
            string: "R",
        },
        TestData {
            fullyEncoded: "%53",
            minimallyEncoded: "S",
            string: "S",
        },
        TestData {
            fullyEncoded: "%54",
            minimallyEncoded: "T",
            string: "T",
        },
        TestData {
            fullyEncoded: "%55",
            minimallyEncoded: "U",
            string: "U",
        },
        TestData {
            fullyEncoded: "%56",
            minimallyEncoded: "V",
            string: "V",
        },
        TestData {
            fullyEncoded: "%57",
            minimallyEncoded: "W",
            string: "W",
        },
        TestData {
            fullyEncoded: "%58",
            minimallyEncoded: "X",
            string: "X",
        },
        TestData {
            fullyEncoded: "%59",
            minimallyEncoded: "Y",
            string: "Y",
        },
        TestData {
            fullyEncoded: "%5A",
            minimallyEncoded: "Z",
            string: "Z",
        },
        TestData {
            fullyEncoded: "%5B",
            minimallyEncoded: "%5B",
            string: "[",
        },
        TestData {
            fullyEncoded: "%5C",
            minimallyEncoded: "%5C",
            string: "\\",
        },
        TestData {
            fullyEncoded: "%5D",
            minimallyEncoded: "%5D",
            string: "]",
        },
        TestData {
            fullyEncoded: "%5E",
            minimallyEncoded: "%5E",
            string: "^",
        },
        TestData {
            fullyEncoded: "%5F",
            minimallyEncoded: "_",
            string: "_",
        },
        TestData {
            fullyEncoded: "%60",
            minimallyEncoded: "%60",
            string: "`",
        },
        TestData {
            fullyEncoded: "%61",
            minimallyEncoded: "a",
            string: "a",
        },
        TestData {
            fullyEncoded: "%62",
            minimallyEncoded: "b",
            string: "b",
        },
        TestData {
            fullyEncoded: "%63",
            minimallyEncoded: "c",
            string: "c",
        },
        TestData {
            fullyEncoded: "%64",
            minimallyEncoded: "d",
            string: "d",
        },
        TestData {
            fullyEncoded: "%65",
            minimallyEncoded: "e",
            string: "e",
        },
        TestData {
            fullyEncoded: "%66",
            minimallyEncoded: "f",
            string: "f",
        },
        TestData {
            fullyEncoded: "%67",
            minimallyEncoded: "g",
            string: "g",
        },
        TestData {
            fullyEncoded: "%68",
            minimallyEncoded: "h",
            string: "h",
        },
        TestData {
            fullyEncoded: "%69",
            minimallyEncoded: "i",
            string: "i",
        },
        TestData {
            fullyEncoded: "%6A",
            minimallyEncoded: "j",
            string: "j",
        },
        TestData {
            fullyEncoded: "%6B",
            minimallyEncoded: "k",
            string: "k",
        },
        TestData {
            fullyEncoded: "%6C",
            minimallyEncoded: "l",
            string: "l",
        },
        TestData {
            fullyEncoded: "%6D",
            minimallyEncoded: "m",
            string: "m",
        },
        TestData {
            fullyEncoded: "%6E",
            minimallyEncoded: "n",
            string: "n",
        },
        TestData {
            fullyEncoded: "%6F",
            minimallyEncoded: "o",
            string: "o",
        },
        TestData {
            fullyEncoded: "%70",
            minimallyEncoded: "p",
            string: "p",
        },
        TestData {
            fullyEncoded: "%71",
            minimallyEncoded: "q",
            string: "q",
        },
        TestData {
            fullyEncoded: "%72",
            minimallyEncoded: "r",
            string: "r",
        },
        TestData {
            fullyEncoded: "%73",
            minimallyEncoded: "s",
            string: "s",
        },
        TestData {
            fullyEncoded: "%74",
            minimallyEncoded: "t",
            string: "t",
        },
        TestData {
            fullyEncoded: "%75",
            minimallyEncoded: "u",
            string: "u",
        },
        TestData {
            fullyEncoded: "%76",
            minimallyEncoded: "v",
            string: "v",
        },
        TestData {
            fullyEncoded: "%77",
            minimallyEncoded: "w",
            string: "w",
        },
        TestData {
            fullyEncoded: "%78",
            minimallyEncoded: "x",
            string: "x",
        },
        TestData {
            fullyEncoded: "%79",
            minimallyEncoded: "y",
            string: "y",
        },
        TestData {
            fullyEncoded: "%7A",
            minimallyEncoded: "z",
            string: "z",
        },
        TestData {
            fullyEncoded: "%7B",
            minimallyEncoded: "%7B",
            string: "{",
        },
        TestData {
            fullyEncoded: "%7C",
            minimallyEncoded: "%7C",
            string: "|",
        },
        TestData {
            fullyEncoded: "%7D",
            minimallyEncoded: "%7D",
            string: "}",
        },
        TestData {
            fullyEncoded: "%7E",
            minimallyEncoded: "~",
            string: "~",
        },
        TestData {
            fullyEncoded: "%7F",
            minimallyEncoded: "%7F",
            string: "\u{007f}",
        },
        TestData {
            fullyEncoded: "%E8%87%AA%E7%94%B1",
            minimallyEncoded: "%E8%87%AA%E7%94%B1",
            string: "\u{81ea}\u{7531}",
        },
        // TODO: how to represent this in rust:
        //  TestData{fullyEncoded: "%F0%90%90%80", minimallyEncoded: "%F0%90%90%80", string: "\ud801\udc00"}
    ];
}
