#[allow(unused)]
#[derive(Debug)]
pub struct Symbol {
    name: String,
    index: usize,
    symbol_type: SymbolType,
}

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType, index: usize) -> Symbol {
        Symbol {
            name,
            symbol_type,
            index,
        }
    }
}

#[derive(Debug)]
pub enum SymbolType {
    Label,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            symbols: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, s: Symbol) {
        self.symbols.push(s);
    }

    pub fn symbol_value(&self, s: &str) -> Option<usize> {
        for symbol in &self.symbols {
            if symbol.name == s {
                return Some(symbol.index);
            }
        }
        None
    }

    pub fn has_symbol(&self, s: &str) -> bool {
        for symbol in &self.symbols {
            if symbol.name == s {
                return true;
            }
        }
        false
    }

    pub fn set_symbol_index(&mut self, s: &str, index: usize) {
        for symbol in &mut self.symbols {
            if symbol.name == s {
                symbol.index = index
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new("test".to_string(), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(true, v.is_some());
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }
}
