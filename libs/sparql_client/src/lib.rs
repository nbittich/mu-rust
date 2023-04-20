use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::Duration;

use mu_rust_common::SessionQueryHeaders;
use mu_rust_common::HEADER_MU_AUTH_SUDO;
use mu_rust_common::HEADER_MU_CALL_ID;
use mu_rust_common::HEADER_MU_SESSION_ID;
use mu_rust_common::SPARQL_ENDPOINT;
use mu_rust_common::SPARQL_RESULT_CONTENT_TYPE;
use new_string_template::template::Template;

use regex::Regex;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
pub use spargebra::Query;
pub use spargebra::Update as UpdateQuery;

pub const REQUEST_TIMEOUT_SECONDS: &str = "REQUEST_TIMEOUT_SECONDS";

const CUSTOM_REGEX: &str = r"(?mi)\$\{([^\}]+)\}";

pub struct SparqlClient {
    reg: Regex,
    client: Client,
    endpoint: String,
}

#[derive(Debug, Default)]
pub struct Config {
    pub endpoint: Option<String>,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SparqlResponse {
    pub head: Head,
    pub results: Option<SparqlResult>,
    pub boolean: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Head {
    pub link: Option<Vec<String>>,
    pub vars: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SparqlResult {
    pub distinct: Option<bool>,
    pub bindings: Vec<BTreeMap<String, Binding>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Binding {
    pub datatype: Option<String>,
    #[serde(rename = "type")]
    pub rdf_type: String,
    pub value: String,
    #[serde(rename = "xml:lang")]
    pub lang: Option<String>,
}

impl SparqlClient {
    pub fn new(config: Config) -> Result<SparqlClient, Box<dyn Error>> {
        let endpoint = if let Some(endpoint) = config.endpoint {
            endpoint
        } else {
            env::var(SPARQL_ENDPOINT).unwrap_or("http://database:8090/sparql".into())
        };
        let timeout = if let Some(timeout) = config.timeout {
            timeout
        } else {
            let timeout = env::var(REQUEST_TIMEOUT_SECONDS).unwrap_or("60".into());
            let timeout = timeout.parse::<u64>()?;
            Duration::from_secs(timeout)
        };
        let reg = Regex::new(CUSTOM_REGEX)?;
        let client = Client::builder()
            .use_rustls_tls()
            .timeout(timeout)
            .build()?;
        Ok(SparqlClient {
            client,
            reg,
            endpoint,
        })
    }
    pub fn make_update_query_from_template(
        &self,
        templ: &str,
        context: &HashMap<&str, String>,
    ) -> Result<UpdateQuery, Box<dyn Error>> {
        let templ = Template::new(templ).with_regex(&self.reg);
        let query = templ.render(context)?;
        dbg!(&query);
        let update_query = UpdateQuery::parse(&query, None)?;
        Ok(update_query)
    }

    pub fn make_query_from_template(
        &self,
        templ: &str,
        context: &HashMap<&str, String>,
    ) -> Result<Query, Box<dyn Error>> {
        let templ = Template::new(templ).with_regex(&self.reg);
        let query = templ.render(context)?;
        println!("{query}");
        let query = Query::parse(&query, None)?;
        Ok(query)
    }

    pub async fn update(
        &self,
        query: UpdateQuery,
        headers: SessionQueryHeaders,
    ) -> Result<(), Box<dyn Error>> {
        let response = self
            .client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, SPARQL_RESULT_CONTENT_TYPE)
            .header(HEADER_MU_CALL_ID, headers.call_id.unwrap_or("".into()))
            .header(
                HEADER_MU_SESSION_ID,
                headers.session_id.unwrap_or("".into()),
            )
            .query(&[
                ("query", query.to_string()),
                ("format", SPARQL_RESULT_CONTENT_TYPE.to_string()),
            ])
            .send()
            .await?;
        let _ = response.error_for_status()?;
        Ok(())
    }
    pub async fn query(
        &self,
        query: Query,
        headers: SessionQueryHeaders,
    ) -> Result<SparqlResponse, Box<dyn Error>> {
        let response = self
            .client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, SPARQL_RESULT_CONTENT_TYPE)
            .header(HEADER_MU_CALL_ID, headers.call_id.unwrap_or("".into()))
            .header(
                HEADER_MU_SESSION_ID,
                headers.session_id.unwrap_or("".into()),
            )
            .query(&[
                ("query", query.to_string()),
                ("format", SPARQL_RESULT_CONTENT_TYPE.to_string()),
            ])
            .send()
            .await?;
        let sparql_result: SparqlResponse = response.json().await?;
        Ok(sparql_result)
    }

    pub async fn update_sudo(&self, query: UpdateQuery) -> Result<(), Box<dyn Error>> {
        let response = self
            .client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, SPARQL_RESULT_CONTENT_TYPE)
            .header(HEADER_MU_AUTH_SUDO, "true")
            .query(&[
                ("query", query.to_string()),
                ("format", SPARQL_RESULT_CONTENT_TYPE.to_string()),
            ])
            .send()
            .await?;
        let _ = response.error_for_status()?;
        Ok(())
    }

    pub async fn query_sudo(&self, query: Query) -> Result<SparqlResponse, Box<dyn Error>> {
        let response = self
            .client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, SPARQL_RESULT_CONTENT_TYPE)
            .header(HEADER_MU_AUTH_SUDO, "true")
            .query(&[
                ("query", query.to_string()),
                ("format", SPARQL_RESULT_CONTENT_TYPE.to_string()),
            ])
            .send()
            .await?;
        let sparql_result: SparqlResponse = response.json().await?;
        Ok(sparql_result)
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use spargebra::{
        algebra::GraphPattern,
        term::{GroundQuadPattern, GroundTermPattern, Literal, NamedNode, TriplePattern, Variable},
        GraphUpdateOperation, Query, Update,
    };

    use crate::SparqlClient;

    #[test]
    fn test_select_query() {
        let client = SparqlClient::new(Default::default()).unwrap();
        let query = client
            .make_query_from_template(
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

        let update_query = client
            .make_update_query_from_template(
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
