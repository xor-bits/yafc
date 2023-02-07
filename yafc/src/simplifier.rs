use crate::ast::{Num, YafcExpr, YafcLanguage};
use egg::{merge_max, rewrite, Analysis, CostFunction, Extractor, Id, Language, Rewrite, Runner};
use once_cell::sync::Lazy;
use std::time::Duration;

//

pub struct Simplifier;

impl Simplifier {
    pub fn run(in_expr: &YafcExpr) -> YafcExpr {
        let runner = Runner::<YafcLanguage, ConstFold>::default()
            .with_time_limit(Duration::from_millis(100))
            .with_expr(in_expr)
            .run(&*RULES);

        let extractor = Extractor::new(&runner.egraph, CostFn);

        let (cost, expr) = extractor.find_best(runner.roots[0]);
        tracing::debug!("cost={cost},expr={}", expr.pretty(50));

        YafcExpr {
            // A hack?:
            root: Some((expr.as_ref().len() - 1).into()),
            expr,
        }
    }
}

//

static RULES: Lazy<Vec<Rewrite<YafcLanguage, ConstFold>>> = Lazy::new(|| {
    let omni_dir = [
        rewrite!("commutative-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("commutative-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("mul-0"; "(* ?a 0)" => "0"),
    ];
    let bi_dir = [
        rewrite!("associative-add"; "(+ ?a (+ ?b ?c))" <=> "(+ (+ ?a ?b) ?c)"),
        rewrite!("associative-mul"; "(* ?a (* ?b ?c))" <=> "(* (* ?a ?b) ?c)"),
        rewrite!("add-0";  "(+ ?a 0)" <=> "?a"),
        rewrite!("add-eq"; "(+ ?a ?a)" <=> "(* 2 ?a)"),
        rewrite!("mul-1";  "(* ?a 1)" <=> "?a"),
        rewrite!("mul-eq"; "(* ?a ?a)" <=> "(^ ?a 2)"),
        rewrite!("combine-like-terms"; "(+ (* ?a ?b) (* ?a ?c))" <=> "(* ?a (+ ?b ?c))"),
    ];
    omni_dir
        .into_iter()
        .chain(bi_dir.into_iter().flatten())
        .collect()
});

//

#[derive(Default)]
struct ConstFold;
impl Analysis<YafcLanguage> for ConstFold {
    type Data = Option<Num>;

    fn make(egraph: &egg::EGraph<YafcLanguage, Self>, enode: &YafcLanguage) -> Self::Data {
        let inner = |i: &Id| egraph[*i].data;
        Some(match enode {
            YafcLanguage::Num(num) => *num,
            YafcLanguage::Add([a, b]) => inner(a)?.checked_add(inner(b)?)?,
            YafcLanguage::Mul([a, b]) => inner(a)?.checked_mul(inner(b)?)?,
            _ => return None,
        })
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> egg::DidMerge {
        merge_max(to, from)
    }

    fn modify(egraph: &mut egg::EGraph<YafcLanguage, Self>, id: Id) {
        if let Some(num) = egraph[id].data {
            let sub = egraph.add(YafcLanguage::Num(num));
            egraph.union(id, sub);
        }
    }
}

struct CostFn;
impl CostFunction<YafcLanguage> for CostFn {
    type Cost = usize;

    fn cost<C>(&mut self, enode: &YafcLanguage, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        let op_cost = match enode {
            YafcLanguage::Var(_) => 2,
            _ => 1,
        };
        enode.fold(op_cost, |sum, i| sum + costs(i))
    }
}
