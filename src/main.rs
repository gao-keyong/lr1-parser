use lr1_parser::parser;

fn main() {
    let mut parser = parser::Parser::new("E");
    parser.add_rule("E'", "E");
    parser.add_rule("E", "E+T");
    parser.add_rule("E", "E-T");
    parser.add_rule("E", "T");
    parser.add_rule("T", "T*F");
    parser.add_rule("T", "T/F");
    parser.add_rule("T", "F");
    parser.add_rule("F", "(E)");
    parser.add_rule("F", "num");
    parser.parse();
}
