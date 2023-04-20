use std::collections::HashMap;
use std::error::Error;

use new_string_template::template::Template;

use regex::Regex;
pub use spargebra::Query;
pub use spargebra::Update as UpdateQuery;

const CUSTOM_REGEX: &str = r"(?mi)\$\{([^\}]+)\}";

pub fn get_update_query(
    templ: &str,
    context: &HashMap<&str, String>,
) -> Result<UpdateQuery, Box<dyn Error>> {
    let reg = Regex::new(CUSTOM_REGEX)?;
    let templ = Template::new(templ).with_regex(&reg);
    let query = templ.render(context)?;
    dbg!(&query);
    let update_query = UpdateQuery::parse(&query, None)?;
    Ok(update_query)
}

pub fn get_query(templ: &str, context: &HashMap<&str, String>) -> Result<Query, Box<dyn Error>> {
    let reg = Regex::new(CUSTOM_REGEX)?;
    let templ = Template::new(templ).with_regex(&reg);
    let query = templ.render(context)?;
    println!("{query}");
    let query = Query::parse(&query, None)?;
    Ok(query)
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use spargebra::{
        algebra::GraphPattern,
        term::{GroundQuadPattern, GroundTermPattern, Literal, NamedNode, TriplePattern, Variable},
        GraphUpdateOperation, Query, Update,
    };

    use crate::{get_query, get_update_query};

    #[test]
    fn test_select_query() {
        let query = get_query(
            include_str!("test_templ/select_query.sparql"),
            &HashMap::from([("bestuurUri", "http://xxx.com/bestuur/x".into())]),
        )
        .unwrap();

        assert_eq!(
            query,
            Query::Select {
                dataset: None,
                base_iri: None,
                pattern: GraphPattern::Project {
                    inner: Box::new(GraphPattern::Bgp {
                        patterns: vec![TriplePattern {
                            subject: spargebra::term::TermPattern::NamedNode(
                                NamedNode::new("http://xxx.com/bestuur/x").unwrap()
                            ),
                            predicate: spargebra::term::NamedNodePattern::Variable(
                                spargebra::term::Variable::new("p").unwrap()
                            ),
                            object: spargebra::term::TermPattern::Variable(
                                spargebra::term::Variable::new("o").unwrap()
                            ),
                        }]
                    }),
                    variables: vec![Variable::new("o").unwrap(), Variable::new("p").unwrap()]
                }
            }
        );

        let update_query = get_update_query(
            include_str!("test_templ/update_query.sparql"),
            &HashMap::from([
                ("someGraph", "http://mygraph.com/public".into()),
                ("someValue", "Hello".into()),
                ("someUri", "http://xxx.com/x".into()),
            ]),
        )
        .unwrap();

        assert_eq!(
            update_query,
            Update {
                base_iri: None,
                operations: vec![GraphUpdateOperation::DeleteInsert {
                    delete: vec![GroundQuadPattern {
                        subject: GroundTermPattern::NamedNode(
                            NamedNode::new("http://xxx.com/x").unwrap()
                        ),
                        predicate: spargebra::term::NamedNodePattern::NamedNode(
                            NamedNode::new("http://some-predicate/pred").unwrap()
                        ),
                        object: GroundTermPattern::Literal(Literal::new_simple_literal("Hello",),),
                        graph_name: spargebra::term::GraphNamePattern::NamedNode(
                            NamedNode::new("http://mygraph.com/public").unwrap()
                        ),
                    },],
                    insert: vec![],
                    using: None,
                    pattern: Box::new(GraphPattern::Graph {
                        name: spargebra::term::NamedNodePattern::NamedNode(
                            NamedNode::new("http://mygraph.com/public").unwrap()
                        ),
                        inner: Box::new(GraphPattern::Bgp {
                            patterns: vec![TriplePattern {
                                subject: spargebra::term::TermPattern::NamedNode(
                                    NamedNode::new("http://xxx.com/x").unwrap()
                                ),
                                predicate: spargebra::term::NamedNodePattern::NamedNode(
                                    NamedNode::new("http://some-predicate/pred").unwrap()
                                ),
                                object: spargebra::term::TermPattern::Literal(
                                    Literal::new_simple_literal("Hello",),
                                ),
                            },],
                        }),
                    }),
                },],
            }
        );
    }
}
