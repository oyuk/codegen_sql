
use crate::parser::Ast;

#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    name: String,
    field_type: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TableStruct {
    name: String,
    fields: Vec<Field>,
}

struct TableStructBuilder {
    name: String,
    fields: Vec<Field>,
}

impl TableStructBuilder {
    fn new() -> Self {
        Self {
            name: "".into(),
            fields: vec![],
        }
    }

    fn name(&mut self, name: String) -> &Self {
        self.name = name;
        self
    }

    fn field(&mut self, field: Field) -> &Self {
        self.fields.push(field);
        self
    }

    fn build(self) -> TableStruct {
        TableStruct {
            name: self.name,
            fields: self.fields,
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn run(&self, ast: Ast) -> TableStruct {
        let mut builder = TableStructBuilder::new();
        self.eval(ast, &mut builder);
        builder.build()
    }

    fn eval(&self, ast: Ast, builder: &mut TableStructBuilder) {
        match ast {
            Ast::Expr { table_name, expr1 } => {
                builder.name(table_name);
                self.eval(*expr1, builder)
            }
            Ast::Expr1 { expr2, expr1 } => {
                self.eval(*expr2, builder);
                if let Some(expr1) = expr1 {
                    self.eval(*expr1, builder)
                }
            }
            Ast::Expr2 {
                name,
                column_type,
                null: _,
            } => {
                builder.field(Field {
                    name,
                    field_type: format!("{:?}", column_type),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::{Field, Interpreter, TableStruct};
    
    use crate::parser::Ast::{Expr, Expr1, Expr2};
    use crate::parser::{ColumnType};

    #[test]
    fn test_run() {
        let interpreter = Interpreter {};
        let ast = Expr {
            table_name: "table_name".into(),
            expr1: Box::new(Expr1 {
                expr2: Box::new(Expr2 {
                    name: "name".into(),
                    column_type: ColumnType::Int,
                    null: true,
                }),
                expr1: None,
            }),
        };
        let result = interpreter.run(ast);
        let fields = vec![Field {
            name: "name".into(),
            field_type: "Int".into(),
        }];
        assert_eq!(
            result,
            TableStruct {
                name: "table_name".into(),
                fields
            }
        )
    }
}
