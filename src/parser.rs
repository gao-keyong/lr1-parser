use crate::rule::*;
use prettytable::{Row, Table};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;

const NTERM_RE_EXPR: &str = r"[A-Z]'?";
const ALPHA_RE_EXPR: &str = r"\+|\-|\*|/|\(|\)|num";

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Item(pub Rule, pub usize, pub Symbol);

type Closure = HashSet<Item>;

#[derive(Debug)]
enum Action {
    Shift(usize),
    Reduce(usize),
    Acc,
}

pub struct Parser {
    rules: Vec<Rule>,
    start_symbol: Symbol,
    nt_set: HashSet<Symbol>,
    alpha_re: Regex,
    first: HashMap<Symbol, HashSet<Symbol>>,
    closures: Vec<Closure>,
    goto: HashMap<(usize, Symbol), usize>,
    action: HashMap<(usize, Symbol), Action>,
}

impl Parser {
    pub fn new(start_symbol: &str) -> Parser {
        Parser {
            rules: vec![],
            start_symbol: Symbol::Nonterminal(start_symbol.to_string()),
            nt_set: HashSet::new(),
            alpha_re: Regex::new(ALPHA_RE_EXPR).unwrap(),
            first: HashMap::new(),
            closures: Vec::new(),
            goto: HashMap::new(),
            action: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, lhs: &str, rhs: &str) {
        let lhs = Symbol::Nonterminal(lhs.to_string());
        self.nt_set.insert(lhs.clone());
        let mut rule = Rule::new(lhs.clone());
        let re = Regex::new(&(NTERM_RE_EXPR.to_owned() + "|" + ALPHA_RE_EXPR)).unwrap();
        let caps_iter = re.captures_iter(rhs);
        for cap in caps_iter {
            if let Some(s) = cap.get(0) {
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

    pub fn list_rules(&self) {
        for (i, rule) in self.rules.iter().enumerate() {
            println!("({}) {}", i, rule);
        }
    }

    fn get_first(&mut self) {
        for nt in &self.nt_set {
            self.first.insert(nt.clone(), HashSet::new());
        }

        loop {
            let prev_first = self.first.clone();

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

    pub fn print_first(&self) {
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

    fn compute_closure(&self, kernel: Closure) -> Closure {
        let mut closure = kernel;

        loop {
            let prev_closure = closure.clone();
            for item in &prev_closure {
                match item.0.rhs.get(item.1) {
                    None => (),
                    Some(s) => {
                        if matches!(s, Symbol::Nonterminal(_)) {
                            let lookaheads = match item.0.rhs.get(item.1 + 1) {
                                None => vec![item.2.clone()],
                                Some(f) => {
                                    if matches!(f, Symbol::Nonterminal(_)) {
                                        self.first.get(f).unwrap().clone().into_iter().collect()
                                    } else {
                                        vec![f.clone()]
                                    }
                                }
                            };
                            for rule in &self.rules {
                                if &rule.lhs == s {
                                    for lookahead in &lookaheads {
                                        closure.insert(Item(rule.clone(), 0, lookahead.clone()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if prev_closure == closure {
                break;
            }
        }
        closure
    }

    fn get_dfa(&mut self) {
        let mut start_cluster = Closure::new();
        start_cluster.insert(Item(
            self.rules.get(0).unwrap().clone(),
            0,
            Symbol::Terminal("$".to_string()),
        ));
        let start_cluster = self.compute_closure(start_cluster);
        self.closures.push(start_cluster);

        let mut index: usize = 0;
        while index < self.closures.len() {
            let edges: HashSet<Symbol> = self.closures[index]
                .iter()
                .filter_map(|item| item.0.rhs.get(item.1))
                .map(|s| s.clone())
                .collect();

            for edge in edges {
                let kernel: Closure = self.closures[index]
                    .iter()
                    .filter(|item| {
                        if let Some(s) = item.0.rhs.get(item.1) {
                            if s == &edge {
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .map(|item| Item(item.0, item.1 + 1, item.2))
                    .collect();

                let closure = self.compute_closure(kernel);

                match self.closures.iter().position(|c| c == &closure) {
                    None => {
                        self.closures.push(closure);
                        self.goto
                            .insert((index, edge.clone()), self.closures.len() - 1);
                    }
                    Some(pos) => {
                        self.goto.insert((index, edge.clone()), pos);
                    }
                }
            }
            index += 1;
        }
    }

    fn get_action(&mut self) {
        for (i, closure) in self.closures.iter().enumerate() {
            for item in closure {
                match item.0.rhs.get(item.1) {
                    Some(s) => {
                        if matches!(s, Symbol::Terminal(_)) {
                            self.action
                                .insert((i, s.clone()), Action::Shift(self.goto[&(i, s.clone())]));
                        }
                    }
                    None => {
                        if item.0.lhs == self.start_symbol {
                            self.action.insert((i, item.2.clone()), Action::Acc);
                        } else {
                            self.action.insert(
                                (i, item.2.clone()),
                                Action::Reduce(
                                    self.rules.iter().position(|r| r == &item.0).unwrap(),
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn print_action(&self) {
        let mut table_t = Table::new();
        const HEAD: [&str; 9] = ["", "+", "-", "*", "/", "(", ")", "num", "$"];
        table_t.add_row(Row::from(HEAD));
        let mut rows: Vec<Vec<String>> = Vec::new();
        for _ in 0..self.closures.len() {
            rows.push(vec!["".to_string(); HEAD.len()]);
        }
        for (key, action) in &self.action {
            let i = key.0;
            let row = rows.get_mut(i).unwrap();
            let column_symbol = &key.1;
            let column_index = HEAD
                .iter()
                .position(|t| {
                    if let Symbol::Terminal(i) = column_symbol {
                        return i == t;
                    } else {
                        false
                    }
                })
                .unwrap();
            row[column_index] = format!("{}", action);
        }
        for (i, row) in rows.iter().enumerate() {
            let mut row = row.clone();
            row[0] = i.to_string();
            table_t.add_row(Row::from(row));
        }
        table_t.printstd();
    }

    pub fn print_goto(&self) {
        let mut table_t = Table::new();
        let head: Vec<_> = self.nt_set.iter().collect();
        let mut head_row: Vec<_> = head.iter().map(|s| s.to_string()).collect();
        head_row.insert(0, "".to_string());
        table_t.add_row(Row::from(head_row));
        for i in 0..self.closures.len() {
            let mut row: Vec<String> = vec![i.to_string()];
            for s in &head {
                let goto = self.goto.get(&(i, Symbol::Nonterminal(s.to_string())));
                match goto {
                    Some(j) => {
                        row.push(j.to_string());
                    }
                    None => {
                        row.push("".to_string());
                    }
                }
            }
            table_t.add_row(Row::from(row));
        }
        table_t.printstd();
    }

    pub fn parse(&mut self) {
        println!("1. 拓广文法：");
        self.list_rules();
        println!("2. 计算FIRST集合：");
        self.get_first();
        self.print_first();
        println!("3. 计算LR(1)项目集规范族和go(I,X)转移函数：");
        self.get_dfa();
        println!(
            "LR(1)项目集规范族共有{}个，DFA的转移边有{}条。",
            self.closures.len(),
            self.goto.len()
        );
        println!("4. 构造LR(1)分析表");
        self.get_action();
        println!("4.1. 表action");
        self.print_action();
        println!("4.2. 表goto");
        self.print_goto();
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Shift(i) => {
                write!(f, "S{}", i);
            }
            Action::Reduce(r) => {
                write!(f, "R{}", r);
            }
            Action::Acc => {
                write!(f, "ACC");
            }
        }
        Ok(())
    }
}
