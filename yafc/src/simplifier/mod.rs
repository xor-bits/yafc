use crate::ast::Ast;

//

mod binary_num_ops;
mod combine_terms;
mod de_paren;
mod factorize;
mod unary_num_ops;

//

pub struct Simplifier;

//

impl Simplifier {
    pub fn run(mut ast: Ast) -> Ast {
        ast = ast.map(32, Self::de_paren);
        ast = ast.map(32, Self::combine_terms);
        ast = ast.map(32, Self::unary_num_ops);
        ast = ast.map(32, Self::binary_num_ops);

        ast
    }
}
