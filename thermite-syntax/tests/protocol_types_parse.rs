use thermite_syntax::{parse, Item, PrimType, Type};

#[test]
fn protocol_declaration_preserves_turns_payloads_and_end() {
    let parsed = parse(
        r#"
protocol PageRequest {
  User { op: u32, count: u64 },
  Provider { status: u32, base: u64 },
  end
}
fn pager(c: PageRequest::Provider) -> ()
  ! blocks
  requires true
  ensures true
{ c.receive(); c.send(0, 0); }
"#,
    );
    assert!(parsed.is_clean(), "unexpected errors: {:?}", parsed.errors);

    let Item::Protocol(protocol) = &parsed.program.items[0] else {
        panic!("expected protocol declaration")
    };
    assert_eq!(protocol.name, "PageRequest");
    assert!(!protocol.repeat);
    assert_eq!(protocol.turns.len(), 2);
    assert_eq!(protocol.turns[0].role, "User");
    assert_eq!(protocol.turns[0].fields[0].name, "op");
    assert_eq!(protocol.turns[0].fields[0].ty, Type::Prim(PrimType::U32));
    assert_eq!(protocol.turns[1].role, "Provider");

    let Item::Fn(function) = &parsed.program.items[1] else {
        panic!("expected function")
    };
    assert_eq!(
        function.params[0].ty,
        Type::ProtocolEndpoint {
            protocol: "PageRequest".into(),
            role: "Provider".into(),
        }
    );
}

#[test]
fn protocol_repeat_tail_is_explicit() {
    let parsed =
        parse("protocol Stream { User { op: u32 }, Provider { value: u64 }, repeat | end }");
    assert!(parsed.is_clean(), "unexpected errors: {:?}", parsed.errors);
    let Item::Protocol(protocol) = &parsed.program.items[0] else {
        panic!("expected protocol declaration")
    };
    assert!(protocol.repeat);
}

#[test]
fn malformed_protocols_recover_to_the_next_item() {
    let parsed = parse(
        "protocol Broken { User { op u32 }, end } fn ok() -> () ! pure requires true ensures true { }",
    );
    assert!(!parsed.errors.is_empty());
    assert!(parsed.program.items.iter().any(|item| item.name() == "ok"));
}
