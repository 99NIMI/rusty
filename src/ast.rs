/// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use std::{fmt::{Debug, Display, Formatter, Result}, unimplemented};

#[derive(PartialEq)]
pub struct POU {
    pub name: String,
    pub variable_blocks: Vec<VariableBlock>,
    pub statements: Vec<Statement>,
    pub pou_type: PouType,
    pub return_type: Option<DataTypeDeclaration>,
    pub linkage : LinkageType,

    pub location: SourceRange,
}

impl Debug for POU {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("POU")
            .field("name", &self.name)
            .field("variable_blocks", &self.variable_blocks)
            .field("statements", &self.statements)
            .field("pou_type", &self.pou_type)
            .field("return_type", &self.return_type)
            .finish()
    }
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum LinkageType {
    Internal,
    External,
}


#[derive(Debug, Copy, PartialEq, Clone)]
pub enum PouType {
    Program,
    Function,
    FunctionBlock,
}

#[derive(Debug, PartialEq)]
pub struct CompilationUnit {
    pub global_vars: Vec<VariableBlock>,
    pub units: Vec<POU>,
    pub types: Vec<UserTypeDeclaration>,
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum VariableBlockType {
    Local,
    Input,
    Output,
    Global,
}

#[derive(PartialEq)]
pub struct VariableBlock {
    pub variables: Vec<Variable>,
    pub variable_block_type: VariableBlockType,
}

impl Debug for VariableBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("VariableBlock")
            .field("variables", &self.variables)
            .field("variable_block_type", &self.variable_block_type)
            .finish()
    }
}

#[derive(PartialEq)]
pub struct Variable {
    pub name: String,
    pub data_type: DataTypeDeclaration,
    pub initializer: Option<Statement>,
    pub location: SourceRange,
}

impl Debug for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.initializer.is_some() {
            f.debug_struct("Variable")
            .field("name", &self.name)
            .field("data_type", &self.data_type)
            .field("initializer", &self.initializer)
            .finish()
        } else {
            f.debug_struct("Variable")
            .field("name", &self.name)
            .field("data_type", &self.data_type)
            .finish()
        }
    }
}

impl Variable {
    pub fn replace_data_type_with_reference_to(
        &mut self,
        type_name: String,
    ) -> DataTypeDeclaration {
        let new_data_type = DataTypeDeclaration::DataTypeReference {
            referenced_type: type_name,
        };
        let old_data_type = std::mem::replace(&mut self.data_type, new_data_type);
        old_data_type
    }
}

pub type SourceRange = core::ops::Range<usize>;

#[derive(Clone, Debug, PartialEq)]
pub struct NewLines {
    new_lines : Vec<usize>,
}

impl NewLines{
    pub fn new(source: &str) -> NewLines {
        let mut new_lines = Vec::new();
        new_lines.push(0);
        for (offset, c) in source.char_indices() {
            if c == '\n' {
                new_lines.push(offset);
            }
        }
        NewLines { new_lines }
    }
    
 /// binary search the first element which is bigger than the given index
 /// starting with line 1
    pub fn get_line_of(&self, offset: usize) -> Option<usize> {

        if offset == 0 { return Some(1); }

        let mut start  = 0;
        let mut end   = self.new_lines.len() - 1;
        let mut result: usize = 0;
        while  start <= end {
            let mid = (start + end) / 2;

            if self.new_lines[mid] <= offset {
                start = mid + 1; //move to the right
            } else {
                result = mid;
                end = mid - 1;
            }
        }

        return if self.new_lines[result] > offset {
            Some(result)
        } else {
            None
        }
    }

    /// get the offset of the new_line that starts line l (starting with line 1)
    pub fn get_offest_of_line(&self, l: usize) -> usize{ 
        self.new_lines[l-1]
    }

    pub fn _get_location_information(&self, offset: &core::ops::Range<usize>) -> String {
        let line = self.get_line_of(offset.start).unwrap_or(1);
        let line_offset = self.get_offest_of_line(line);
        let offset = offset.start-line_offset .. offset.end-line_offset;
        format!("line: {:}, offset: {:?}", line, offset)
    }
}

#[derive(Debug, PartialEq)]
pub enum DataTypeDeclaration {
    DataTypeReference { referenced_type: String },
    DataTypeDefinition { data_type: DataType },
}

impl DataTypeDeclaration {
    pub fn get_name<'ctx>(&'ctx self) -> Option<&'ctx str> {
        match self {
            DataTypeDeclaration::DataTypeReference { referenced_type } => {
                Some(referenced_type.as_str())
            }
            DataTypeDeclaration::DataTypeDefinition { data_type } => data_type.get_name(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct UserTypeDeclaration {
    pub data_type: DataType,
    pub initializer: Option<Statement>,
}

#[derive(PartialEq)]
pub enum DataType {
    StructType {
        name: Option<String>, //maybe None for inline structs
        variables: Vec<Variable>,
    },
    EnumType {
        name: Option<String>, //maybe empty for inline enums
        elements: Vec<String>,
    },
    SubRangeType {
        name: Option<String>,
        referenced_type: String,
    },
    ArrayType {
        name: Option<String>,
        bounds: Statement,
        referenced_type: Box<DataTypeDeclaration>,
    },
}

impl Debug for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DataType::StructType { name, variables } => f
                .debug_struct("StructType")
                .field("name", name)
                .field("variables", variables)
                .finish(),
            DataType::EnumType { name, elements } => f
                .debug_struct("EnumType")
                .field("name", name)
                .field("elements", elements)
                .finish(),
            DataType::SubRangeType {
                name,
                referenced_type,
            } => f
                .debug_struct("SubRangeType")
                .field("name", name)
                .field("referenced_type", referenced_type)
                .finish(),
            DataType::ArrayType {
                name,
                bounds,
                referenced_type,
            } => f
                .debug_struct("ArrayType")
                .field("name", name)
                .field("bounds", bounds)
                .field("referenced_type", referenced_type)
                .finish(),
        }
    }
}

impl DataType {
    pub fn set_name(&mut self, new_name: String) {
        match self {
            DataType::StructType { name, variables: _ } => *name = Some(new_name),
            DataType::EnumType { name, elements: _ } => *name = Some(new_name),
            DataType::SubRangeType {
                name,
                ..
            } => *name = Some(new_name),
            DataType::ArrayType { name, .. } => *name = Some(new_name),
        }
    }

    pub fn get_name<'ctx>(&'ctx self) -> Option<&'ctx str> {
        match self {
            DataType::StructType { name, variables: _ } => name.as_ref().map(|x| x.as_str()),
            DataType::EnumType { name, elements: _ } => name.as_ref().map(|x| x.as_str()),
            DataType::SubRangeType {
                name,
                ..
            } => name.as_ref().map(|x| x.as_str()),
            DataType::ArrayType { name, .. } => name.as_ref().map(|x| x.as_str()),
        }
    }

    //Attempts to replace the inner type with a reference. Returns the old type if replaceable
    pub fn replace_data_type_with_reference_to(
        &mut self,
        type_name: String,
    ) -> Option<DataTypeDeclaration> {
        if let DataType::ArrayType{referenced_type,..} = self {
            if let DataTypeDeclaration::DataTypeReference{..} = **referenced_type {
                return None;
            } 
            let new_data_type = DataTypeDeclaration::DataTypeReference {
                referenced_type: type_name,
            };
            let old_data_type = std::mem::replace(referenced_type, Box::new(new_data_type));
            Some(*old_data_type)
        } else {
            None
        }
    }
}

#[derive(PartialEq)]
pub struct ConditionalBlock {
    pub condition: Box<Statement>,
    pub body: Vec<Statement>,
}

impl Debug for ConditionalBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("ConditionalBlock")
            .field("condition", &self.condition)
            .field("body", &self.body)
            .finish()
    }
}

#[derive(PartialEq)]
pub enum Statement {
    // Literals
    LiteralInteger {
        value: String,
        location: SourceRange,
    },
    LiteralReal {
        value: String,
        location: SourceRange,
    },
    LiteralBool {
        value: bool,
        location: SourceRange,
    },
    LiteralString {
        value: String,
        location: SourceRange,
    },
    LiteralArray {
        elements: Option<Box<Statement>>,    // expression-list
        location: SourceRange,
    },
    // Expressions
    QualifiedReference {
        elements: Vec<Statement>
    },
    Reference {
        name: String,
        location: SourceRange,
    },
    ArrayAccess {
        reference: Box<Statement>,
        access: Box<Statement>
    },
    BinaryExpression {
        operator: Operator,
        left: Box<Statement>,
        right: Box<Statement>,
    },
    UnaryExpression {
        operator: Operator,
        value: Box<Statement>,
        location: SourceRange,
    },
    ExpressionList {
        expressions: Vec<Statement>,
    },
    RangeStatement {
        start: Box<Statement>,
        end: Box<Statement>,
    },
    // Assignment
    Assignment {
        left: Box<Statement>,
        right: Box<Statement>,
    },
    // OutputAssignment
    OutputAssignment {
        left: Box<Statement>,
        right: Box<Statement>,
    },
    //Call Statement
    CallStatement {
        operator: Box<Statement>,
        parameters: Box<Option<Statement>>,
        location: SourceRange,
    },
    // Control Statements
    IfStatement {
        blocks: Vec<ConditionalBlock>,
        else_block: Vec<Statement>,
        location: SourceRange,
    },
    ForLoopStatement {
        counter: Box<Statement>,
        start: Box<Statement>,
        end: Box<Statement>,
        by_step: Option<Box<Statement>>,
        body: Vec<Statement>,
        location: SourceRange,
    },
    WhileLoopStatement {
        condition: Box<Statement>,
        body: Vec<Statement>,
        location: SourceRange,
    },
    RepeatLoopStatement {
        condition: Box<Statement>,
        body: Vec<Statement>,
        location: SourceRange,
    },
    CaseStatement {
        selector: Box<Statement>,
        case_blocks: Vec<ConditionalBlock>,
        else_block: Vec<Statement>,
        location: SourceRange,
    },
}

impl Debug for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Statement::LiteralInteger { value, .. } => f
                .debug_struct("LiteralInteger")
                .field("value", value)
                .finish(),
            Statement::LiteralReal { value, .. } => {
                f.debug_struct("LiteralReal").field("value", value).finish()
            }
            Statement::LiteralBool { value, .. } => {
                f.debug_struct("LiteralBool").field("value", value).finish()
            }
            Statement::LiteralString { value, .. } => f
                .debug_struct("LiteralString")
                .field("value", value)
                .finish(),
            Statement::LiteralArray { elements, .. } => f
                .debug_struct("LiteralArray")
                .field("elements", elements)
                .finish(),
            Statement::Reference { name, .. } => f
                .debug_struct("Reference")
                .field("name", name)
                .finish(),
            Statement::QualifiedReference {elements, ..} => f
                .debug_struct("QualifiedReference")
                .field("elements", elements)
                .finish(),
            Statement::BinaryExpression {
                operator,
                left,
                right,
                ..
            } => f
                .debug_struct("BinaryExpression")
                .field("operator", operator)
                .field("left", left)
                .field("right", right)
                .finish(),
            Statement::UnaryExpression {
                operator, value, ..
            } => f
                .debug_struct("UnaryExpression")
                .field("operator", operator)
                .field("value", value)
                .finish(),
            Statement::ExpressionList { expressions } => f
                .debug_struct("ExpressionList")
                .field("expressions", expressions)
                .finish(),
            Statement::RangeStatement { start, end } => f
                .debug_struct("RangeStatement")
                .field("start", start)
                .field("end", end)
                .finish(),
            Statement::Assignment { left, right } => f
                .debug_struct("Assignment")
                .field("left", left)
                .field("right", right)
                .finish(),
            Statement::OutputAssignment { left, right } => f
                .debug_struct("OutputAssignment")
                .field("left", left)
                .field("right", right)
                .finish(),
            Statement::CallStatement {
                operator,
                parameters,
                ..
            } => f
                .debug_struct("CallStatement")
                .field("operator", operator)
                .field("parameters", parameters)
                .finish(),
            Statement::IfStatement {
                blocks, else_block, ..
            } => f
                .debug_struct("IfStatement")
                .field("blocks", blocks)
                .field("else_block", else_block)
                .finish(),
            Statement::ForLoopStatement {
                counter,
                start,
                end,
                by_step,
                body,
                ..
            } => f
                .debug_struct("ForLoopStatement")
                .field("counter", counter)
                .field("start", start)
                .field("end", end)
                .field("by_step", by_step)
                .field("body", body)
                .finish(),
            Statement::WhileLoopStatement {
                condition, body, ..
            } => f
                .debug_struct("WhileLoopStatement")
                .field("condition", condition)
                .field("body", body)
                .finish(),
            Statement::RepeatLoopStatement {
                condition, body, ..
            } => f
                .debug_struct("RepeatLoopStatement")
                .field("condition", condition)
                .field("body", body)
                .finish(),
            Statement::CaseStatement {
                selector,
                case_blocks,
                else_block,
                ..
            } => f
                .debug_struct("CaseStatement")
                .field("selector", selector)
                .field("case_blocks", case_blocks)
                .field("else_block", else_block)
                .finish(),
            Statement::ArrayAccess {
                reference, access, ..
            } => f
                .debug_struct("ArrayAccess")
                .field("reference", reference)
                .field("access", access)
                .finish(),
        }
    }
}

impl Statement {
    ///Returns the statement in a singleton list, or the contained statements if the statement is already a list
    pub fn get_as_list(&self) -> Vec<&Statement> {
        if let Statement::ExpressionList{expressions} = self {
            expressions.iter().collect::<Vec<&Statement>>()
        } else {
            vec![self]
        }

    }
    pub fn get_location(&self) -> SourceRange {
        match self {
            Statement::LiteralInteger { location, .. } => location.clone(),
            Statement::LiteralReal { location, .. } => location.clone(),
            Statement::LiteralBool { location, .. } => location.clone(),
            Statement::LiteralString { location, .. } => location.clone(),
            Statement::LiteralArray {location, ..} => location.clone(),
            Statement::Reference { location, .. } => location.clone(),
            Statement::QualifiedReference { elements, .. } => {
                elements.first().map_or(0, |it| it.get_location().start)
                    ..elements.last().map_or(0, |it|it.get_location().end)
            }
            Statement::BinaryExpression { left, right, .. } => {
                left.get_location().start..right.get_location().end
            }
            Statement::UnaryExpression { location, .. } => location.clone(),
            Statement::ExpressionList { expressions } => {
                expressions.first().map_or(0, |it| it.get_location().start)
                    ..expressions.last().map_or(0, |it| it.get_location().end)
            }
            Statement::RangeStatement { start, end } => {
                start.get_location().start..end.get_location().end
            }
            Statement::Assignment { left, right } => {
                left.get_location().start..right.get_location().end
            }
            Statement::OutputAssignment { left, right } => {
                left.get_location().start..right.get_location().end
            }
            Statement::CallStatement { location, .. } => location.clone(),
            Statement::IfStatement { location, .. } => location.clone(),
            Statement::ForLoopStatement { location, .. } => location.clone(),
            Statement::WhileLoopStatement { location, .. } => location.clone(),
            Statement::RepeatLoopStatement { location, .. } => location.clone(),
            Statement::CaseStatement { location, .. } => location.clone(),
            Statement::ArrayAccess { reference, access } => {
                reference.get_location().start..access.get_location().end
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiplication,
    Division,
    Equal,
    NotEqual,
    Modulo,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    Not,
    And,
    Or,
    Xor,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let symbol = match self {
            Operator::Plus=>  "+",
            Operator::Minus =>  "-",
            Operator::Multiplication =>  "*",
            Operator::Division =>  "/",
            Operator::Equal =>  "=",
            _ =>  unimplemented!(),
        };
        f.write_str(symbol)
    }
}

pub fn flatten_expression_list(condition: &Statement) -> Vec<&Statement> {
    match condition {
        Statement::ExpressionList{ expressions} => expressions.iter().by_ref().collect(),
        _ => vec![condition]
    }
}
