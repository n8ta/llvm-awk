use std::env::var;
use immutable_chunkmap::map::Map;
use crate::{Expr};
use crate::parser::{AwkT, Stmt, TypedExpr};

pub type MapT = Map<String, AwkT, 1000>;

pub fn analyze(stmt: &mut Stmt) {
    let mut map = MapT::new();
    TypeAnalysis { map }.analyze_stmt(stmt)
}

struct TypeAnalysis {
    map: MapT,
}

impl TypeAnalysis {
    pub fn analyze_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Expr(expr) => self.analyze_expr(expr),
            Stmt::Print(expr) => self.analyze_expr(expr),
            Stmt::Assign(var, value) => {
                self.analyze_expr(value);
                self.map = self.map.insert(var.clone(), value.typ).0;
            }
            Stmt::Group(grouping) => {
                for stmt in grouping {
                    self.analyze_stmt(stmt);
                }
            }
            Stmt::If(test, if_so, if_not) => {
                self.analyze_expr(test);
                let mut if_so_map = self.map.clone();
                let mut if_not_map = self.map.clone();
                std::mem::swap(&mut if_so_map, &mut self.map);

                self.analyze_stmt(if_so);
                std::mem::swap(&mut if_so_map, &mut self.map);
                std::mem::swap(&mut if_not_map, &mut self.map);
                if let Some(else_case) = if_not {
                    self.analyze_stmt(else_case)
                }
                std::mem::swap(&mut if_not_map, &mut self.map);
                self.map = TypeAnalysis::merge_maps(&[&if_so_map, &if_not_map]);
            }
            Stmt::While(test, body) => {
                self.analyze_expr(test);

                self.analyze_stmt(body);

                self.analyze_expr(test);
                self.analyze_stmt(body);
            }
        }
    }

    pub fn analyze_expr(&self, expr: &mut TypedExpr) {
        match &mut expr.expr {
            Expr::NumberF64(_) => {
                expr.typ = AwkT::Float;
            }
            Expr::String(_) => {
                expr.typ = AwkT::String;
            }
            Expr::BinOp(left, op, right) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
                expr.typ = AwkT::Float;

            }
            Expr::MathOp(left, op, right) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
                expr.typ = AwkT::Float;
            }
            Expr::LogicalOp(left, op, right) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
                expr.typ = AwkT::Float;
            }
            Expr::Variable(var) => {
                if let Some(typ) = self.map.get(var) {
                    expr.typ = *typ;
                } else {
                    expr.typ = AwkT::Float;
                }
            }
            Expr::Column(col) => {
                expr.typ = AwkT::String;
                self.analyze_expr(col);
            }
            Expr::Call => {
                expr.typ = AwkT::Float
            }
        }
    }

    fn merge_maps(children: &[&MapT]) -> MapT {
        let mut merged = MapT::new();
        for map in children {
            for (name, var_type)  in map.into_iter() {
                if let Some(existing_type) = merged.get(name) {
                    merged = merged.insert(name.clone(), TypeAnalysis::merge_types(existing_type, var_type)).0;
                } else {
                    merged = merged.insert(name.clone(), *var_type).0;
                }
            }
        }
        merged

        // let mut merged = map_a.clone();
        // for x in map_b {
        //     if let Some(map_a_val) = map_a.get(x.0) {
        //         let merged_type = TypeAnalysis::merge_types(&x.1, map_a_val);
        //         merged = merged.insert(x.0.clone(), merged_type).0;
        //     }
        // }
        // merged
    }
    fn merge_types(a: &AwkT, b: &AwkT) -> AwkT {
        match (a, b) {
            (AwkT::Float, AwkT::Float) => AwkT::Float,
            (AwkT::String, AwkT::String) => AwkT::String,
            _ => AwkT::Variable,
        }
    }
}

#[cfg(test)]
fn test_it(program: &str, expected: &str) {
    fn strip(data: &str) -> String {
        data.replace("\n", "").replace(" ", "").replace("\t", "").replace(";", "")
    }

    use crate::{lex, parse, transform};
    let mut ast = transform(parse(lex(program).unwrap()));
    analyze(&mut ast);
    println!("prog: {:?}", ast);
    let result_clean = strip(&format!("{}", ast));
    let expected_clean = strip(expected);
    if result_clean != expected_clean {
        println!("Got: \n{}", format!("{}", ast));
        println!("Expected: \n{}", expected);
    }
    assert_eq!(result_clean, expected_clean);
}

#[test]
fn test_typing_basic() {
    test_it("BEGIN { print \"a\" }", "print (s \"a\")");
}

#[test]
fn test_typing_basic2() {
    test_it("BEGIN { print 123 }", "print (f 123)");
}

#[test]
fn test_if_basic() {
    test_it("BEGIN { a = 1; print a; if($1) { print a } } ",
            "a = (f 1); print (f a); if (s $(f 1)) { print (f a) }");
}

#[test]
fn test_if_polluting() {
    test_it("BEGIN { a = 1; print a; if($1) { a = \"a\"; } print a; print a;    } ",
            "a = (f 1); print (f a); if (s $(f 1)) { a = (s \"a\"); } print (v a); print (v a)");
}

#[test]
fn test_if_nonpolluting() {
    test_it("BEGIN { a = 1; print a; if($1) { a = 5; } print a; } ",
            "a = (f 1); print (f a); if (s $(f 1)) { a = (f 5); } print (f a);");
}

#[test]
fn test_ifelse_polluting() {
    test_it("BEGIN { a = 1; print a; if($1) { a = 5; } else { a = \"a\" } print a; } ",
            "a = (f 1); print (f a); if (s $(f 1)) { a = (f 5); } else { a = (s \"a\") } print (v a);");
}

#[test]
fn test_ifelse_swapping() {
    test_it("BEGIN { a = 1; print a; if($1) { a = \"a\"; } else { a = \"a\" } print a; } ",
            "a = (f 1); print (f a); if (s $(f 1)) { a = (s \"a\"); } else { a = (s \"a\") } print (s a);");
}
#[test]
fn test_ifelse_swapping_2() {
    test_it("BEGIN { a = \"a\"; print a; if($1) { a = 3; } else { a = 4 } print a; } ",
            "a = (s \"a\"); print (s a); if (s $(f 1)) { a = ( f 3); } else { a = (f 4) } print (f a);");
}

#[test]
fn test_if_else_polluting() {
    test_it("BEGIN { a = 1; print a; if($1) { a = \"a\"; } else { a = \"a\" } print a; } ",
            "a = (f 1); print (f a); if (s $(f 1)) { a = (s \"a\"); } else { a = (s \"a\"); } print (s a)");
}