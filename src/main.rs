use lr1_parser::parser;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    /// 需要分析的表达式
    #[structopt(help = "Expression to be parsed")]
    expr: String,
}

fn main() {
    let opt = Opt::from_args();
    let expr = opt.expr;
    let mut parser = parser::Parser::new("E'");
    parser.add_rule("E'", "E");
    parser.add_rule("E", "E+T");
    parser.add_rule("E", "E-T");
    parser.add_rule("E", "T");
    parser.add_rule("T", "T*F");
    parser.add_rule("T", "T/F");
    parser.add_rule("T", "F");
    parser.add_rule("F", "(E)");
    parser.add_rule("F", "num");
    parser.parse(&expr);
}
