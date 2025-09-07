use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "engine/lang/workflow.pest"]
pub struct WorkflowParser;

pub fn parse_workflow(
    input: &str
) -> Result<pest::iterators::Pairs<Rule>, pest::error::Error<Rule>> {
    WorkflowParser::parse(Rule::program, input)
}
