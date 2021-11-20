use ::std::fmt;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Symbol {
    Nonterminal(String),
    Terminal(String),
}

impl Symbol {
    pub fn to_string(&self) -> String {
        match self {
            Symbol::Nonterminal(s) => s.to_string(),
            Symbol::Terminal(s) => s.to_string(),
        }
    }
}

pub struct Rule {
    pub lhs: Symbol,
    pub rhs: Vec<Symbol>,
}

impl Rule {
    pub fn new(lhs: Symbol) -> Rule {
        Rule { lhs, rhs: vec![] }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Symbol::Nonterminal(s) => write!(f, "{}", s),
            Symbol::Terminal(s) => {
                if s.is_empty() {
                    write!(f, "ε")
                } else {
                    write!(f, "{}", s)
                }
            }
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}->", self.lhs);
        for s in &self.rhs {
            write!(f, "{}", s);
        }
        Ok(())
    }
}
