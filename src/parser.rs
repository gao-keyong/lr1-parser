use crate::rule::*;
use regex::Regex;
use std::collections::{HashSet,HashMap};
use prettytable::{Table, Row};

const NTERM_RE_EXPR:&str=r"[A-Z]'?";
const ALPHA_RE_EXPR:&str=r"\+|\-|\*|/|\(|\)|num";

pub struct Parser{
    rules: Vec<Rule>,
    start_symbol: Symbol,
    nt_set: HashSet<Symbol>,
    alpha_re: Regex,
    first: HashMap<Symbol, HashSet<Symbol>>,
}

impl Parser{
    pub fn new(start_symbol: &str) -> Parser{
        Parser{
            rules: vec![],
            start_symbol:Symbol::Nonterminal(start_symbol.to_string()),
            nt_set:HashSet::new(),
            alpha_re:Regex::new(ALPHA_RE_EXPR).unwrap(),
            first:HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, lhs: &str, rhs: &str){
        let lhs=Symbol::Nonterminal(lhs.to_string());
        self.nt_set.insert(lhs.clone());
        let mut rule=Rule::new(lhs.clone());
        let re=Regex::new(&(NTERM_RE_EXPR.to_owned()+"|"+ALPHA_RE_EXPR)).unwrap();
        let caps_iter = re.captures_iter(rhs);
        for cap in caps_iter{
            if let Some(s)=cap.get(0){
                let s = s.as_str();
                if self.alpha_re.is_match(s) {
                    rule.rhs.push(Symbol::Terminal(s.to_string()));
                } else {
                    rule.rhs.push(Symbol::Nonterminal(s.to_string()));
                }
            }
        }
        self.rules.push(rule);
    }

    pub fn list_rules(&self){
        for (i,rule) in self.rules.iter().enumerate(){
            println!("({}) {}",i,rule);
        }
    }

    fn get_first(&mut self){
        for nt in &self.nt_set{
            self.first.insert(nt.clone(), HashSet::new());
        }

        loop{
            let prev_first=self.first.clone();

            for rule in &self.rules {
                let left = &rule.lhs;
                let right = &rule.rhs;
                match right.get(0) {
                    Some(symbol) => {
                        let first = self.first.get_mut(left).unwrap();
                        if matches!(symbol, Symbol::Terminal(_)) {
                            first.insert(symbol.clone());
                        } else {
                            first.extend(
                                prev_first
                                    .get(symbol)
                                    .unwrap()
                                    .into_iter()
                                    .map(|i| i.clone()),
                            );
                        }
                    }
                    None => panic!(),
                }
            }
            if prev_first == self.first {
                break;
            }
        }
    }

    pub fn print_first(&self){
        let mut table_t = Table::new();
        table_t.add_row(Row::from(["", "FIRST"]));
        for (key, set) in &self.first {
            let mut first_ele = String::new();
            for symbol in set {
                first_ele += &format!("{} ", symbol);
            }
            let row = [format!("{}", key), first_ele];
            table_t.add_row(Row::from(row));
        }
        table_t.printstd();
    }

    pub fn parse(&mut self){
        println!("1. 拓广文法：");
        self.list_rules();
        println!("2. 计算FIRST集合：");
        self.get_first();
        self.print_first();
        println!("3. 计算LR(1)项目集规范族和go(I,X)转移函数：")
    }
}