use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum ListType {
    Bulleted,
    Numbered,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum SectionBreakType {
    NextPage,
    Continuous,
    EvenPage,
    OddPage,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum ImagePlacement {
    Inline,
    Anchored,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum SearchMode {
    Exact,
    Substring,
    Regex,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum MergeDirection {
    Horizontal,
    Vertical,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum HeaderFooterType {
    DefaultHeader,
    DefaultFooter,
    FirstPageHeader,
    FirstPageFooter,
    EvenPageHeader,
    EvenPageFooter,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum BatchOperationType {
    InsertParagraph,
    ReplaceText,
    DeleteContent,
    InsertRun,
    UpdateParagraphText,
}
